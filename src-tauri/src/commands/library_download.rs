use std::io::Read;
use std::path::{Path, PathBuf};
use std::sync::atomic::Ordering;
use tauri::{Emitter, State};
use crate::app_state::AppState;
use std::fs;

const SYMBOLS_URL: &str = "https://github.com/KiCad/kicad-symbols/archive/refs/heads/master.zip";
const FOOTPRINTS_URL: &str = "https://github.com/KiCad/kicad-footprints/archive/refs/heads/master.zip";

#[derive(Debug, Clone, serde::Serialize)]
pub struct DownloadProgress {
    pub phase: String,
    pub current: usize,
    pub total: usize,
    pub message: String,
}

#[tauri::command]
pub async fn download_kicad_libraries(state: State<'_, AppState>) -> Result<String, String> {
    let app_handle = state.app_handle.read().clone();
    let temp_dir = std::env::temp_dir().join("tracecraft_kicad_download");
    if temp_dir.exists() {
        fs::remove_dir_all(&temp_dir).map_err(|e| e.to_string())?;
    }
    fs::create_dir_all(&temp_dir).map_err(|e| e.to_string())?;

    let emit = |phase: &str, current: usize, total: usize, message: &str| {
        if let Some(ref handle) = app_handle {
            let _ = handle.emit("kicad-download-progress", DownloadProgress {
                phase: phase.to_string(),
                current,
                total,
                message: message.to_string(),
            });
        }
    };

    emit("download", 0, 4, "Starting KiCad library download...");

    let symbols_zip = temp_dir.join("kicad-symbols.zip");
    {
        emit("download", 0, 4, "Downloading symbols library (~60MB)...");
        let resp = ureq::get(SYMBOLS_URL)
            .call()
            .map_err(|e| format!("Failed to download symbols: {}", e))?;
        let mut buf = Vec::new();
        let mut reader = resp.into_body().into_reader();
        reader.read_to_end(&mut buf).map_err(|e| e.to_string())?;
        fs::write(&symbols_zip, &buf).map_err(|e| e.to_string())?;
    }

    let footprints_zip = temp_dir.join("kicad-footprints.zip");
    {
        emit("download", 1, 4, "Downloading footprints library (~300MB, this may take a while)...");
        let resp = ureq::get(FOOTPRINTS_URL)
            .call()
            .map_err(|e| format!("Failed to download footprints: {}", e))?;
        let mut buf = Vec::new();
        let mut reader = resp.into_body().into_reader();
        reader.read_to_end(&mut buf).map_err(|e| e.to_string())?;
        fs::write(&footprints_zip, &buf).map_err(|e| e.to_string())?;
    }

    emit("download", 2, 4, "Downloads complete, extracting...");

    let symbols_dir = temp_dir.join("kicad-symbols");
    extract_zip(&symbols_zip, &symbols_dir).map_err(|e| format!("Failed to extract symbols: {}", e))?;

    let footprints_dir = temp_dir.join("kicad-footprints");
    extract_zip(&footprints_zip, &footprints_dir).map_err(|e| format!("Failed to extract footprints: {}", e))?;

    emit("download", 3, 4, "Extraction complete. Importing into library...");

    let repo = state.library.clone();
    let mut total_imported = 0;

    let sym_files = walk_kicad_sym_files(&symbols_dir);
    let total_sym = sym_files.len();
    emit("import_symbols", 0, total_sym, &format!("Importing {} symbol files...", total_sym));

    for (i, path) in sym_files.iter().enumerate() {
        if state.routing_cancelled.load(Ordering::SeqCst) {
            emit("cancelled", i, total_sym, "Cancelled by user");
            return Err("Download cancelled".to_string());
        }
        let path_str = path.to_string_lossy().to_string();
        match crate::io::library_import::import_kicad_sym_lib(&path_str) {
            Ok(components) => {
                for comp in components {
                    if let Err(e) = repo.save(&comp).await {
                        tracing::warn!("Failed to save component from {}: {}", path.display(), e);
                    } else {
                        total_imported += 1;
                    }
                }
            }
            Err(e) => {
                tracing::warn!("Failed to import symbol file {}: {}", path.display(), e);
            }
        }
        if i % 10 == 0 || i == total_sym - 1 {
            emit("import_symbols", i + 1, total_sym, &format!("Imported symbols: {}/{}", i + 1, total_sym));
        }
    }

    emit("import", 1, 3, "Symbols imported. Importing footprints...");

    let footprint_dirs = walk_kicad_pretty_dirs(&footprints_dir);
    let total_fp = footprint_dirs.len();
    emit("import_footprints", 0, total_fp, &format!("Importing {} footprint libraries...", total_fp));

    for (i, dir) in footprint_dirs.iter().enumerate() {
        if state.routing_cancelled.load(Ordering::SeqCst) {
            emit("cancelled", i, total_fp, "Cancelled by user");
            return Err("Download cancelled".to_string());
        }
        let dir_str = dir.to_string_lossy().to_string();
        match crate::io::library_import::import_kicad_pretty_lib(&dir_str) {
            Ok(components) => {
                for comp in components {
                    if let Err(e) = repo.save(&comp).await {
                        tracing::warn!("Failed to save component from {}: {}", dir.display(), e);
                    } else {
                        total_imported += 1;
                    }
                }
            }
            Err(e) => {
                tracing::warn!("Failed to import footprint dir {}: {}", dir.display(), e);
            }
        }
        if i % 5 == 0 || i == total_fp - 1 {
            emit("import_footprints", i + 1, total_fp, &format!("Imported footprints: {}/{}", i + 1, total_fp));
        }
    }

    emit("done", total_imported, total_imported, &format!("Import complete! {} components imported.", total_imported));

    let _ = fs::remove_dir_all(&temp_dir);

    Ok(format!("Successfully imported {} components from KiCad libraries", total_imported))
}

fn extract_zip(zip_path: &Path, out_dir: &Path) -> Result<(), String> {
    let f = fs::File::open(zip_path).map_err(|e| e.to_string())?;
    let mut archive = zip::ZipArchive::new(f).map_err(|e| e.to_string())?;
    for i in 0..archive.len() {
        let mut entry = archive.by_index(i).map_err(|e| e.to_string())?;
        let name = entry.name().to_string();
        let path = out_dir.join(&name);
        if entry.is_dir() {
            fs::create_dir_all(&path).map_err(|e| e.to_string())?;
        } else {
            if let Some(parent) = path.parent() {
                fs::create_dir_all(parent).map_err(|e| e.to_string())?;
            }
            let mut out = fs::File::create(&path).map_err(|e| e.to_string())?;
            std::io::copy(&mut entry, &mut out).map_err(|e| e.to_string())?;
        }
    }
    Ok(())
}

fn walk_kicad_sym_files(dir: &Path) -> Vec<PathBuf> {
    let mut files = Vec::new();
    if let Ok(entries) = fs::read_dir(dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() {
                files.extend(walk_kicad_sym_files(&path));
            } else if path.extension().map_or(false, |e| e == "kicad_sym") {
                files.push(path);
            }
        }
    }
    files
}

fn walk_kicad_pretty_dirs(dir: &Path) -> Vec<PathBuf> {
    let mut dirs = Vec::new();
    if let Ok(entries) = fs::read_dir(dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() {
                if path.extension().map_or(false, |e| e == "pretty") {
                    dirs.push(path);
                } else {
                    dirs.extend(walk_kicad_pretty_dirs(&path));
                }
            }
        }
    }
    dirs
}
