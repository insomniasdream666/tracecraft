use tauri::State;
use crate::app_state::AppState;
use crate::model::project::*;
use crate::model::pcb::PcbBoard;
use crate::model::schematic::SchematicSheet;
use crate::model::id::SheetId;

fn store_last_project(db: &rusqlite::Connection, path: &str) -> Result<(), String> {
    db.execute(
        "CREATE TABLE IF NOT EXISTS app_settings (key TEXT PRIMARY KEY, value TEXT NOT NULL)",
        [],
    ).map_err(|e| e.to_string())?;
    db.execute(
        "INSERT OR REPLACE INTO app_settings (key, value) VALUES ('last_project_path', ?1)",
        rusqlite::params![path],
    ).map_err(|e| e.to_string())?;
    Ok(())
}

#[tauri::command]
pub fn create_project(state: State<AppState>, name: String) -> Result<ProjectSummary, String> {
    let id = uuid::Uuid::new_v4();
    let now = chrono::Utc::now().to_rfc3339();
    let project = Project {
        id,
        name: name.clone(),
        path: String::new(),
        created_at: now.clone(),
        updated_at: now.clone(),
    };

    let sheet = SchematicSheet {
        id: SheetId(1),
        name: "Sheet 1".to_string(),
        ..Default::default()
    };
    let mut schematics = state.schematics.write();
    schematics.insert(SheetId(1), sheet);
    drop(schematics);

    *state.project.write() = Some(project);
    Ok(ProjectSummary {
        id,
        name,
        path: String::new(),
        updated_at: now,
    })
}

#[tauri::command]
pub fn open_project(state: State<AppState>, path: String) -> Result<ProjectSummary, String> {
    let (project, board, schematics) = crate::io::native::load(&path)?;
    let summary = ProjectSummary {
        id: project.id,
        name: project.name.clone(),
        path: project.path.clone(),
        updated_at: project.updated_at.clone(),
    };

    *state.project.write() = Some(project);
    *state.board.write() = board;
    state.schematics.write().extend(schematics);

    if let Ok(db) = state.db.lock() {
        let _ = store_last_project(&db, &path);
    }

    Ok(summary)
}

#[tauri::command]
pub fn save_project(state: State<AppState>, path: String) -> Result<(), String> {
    let project = state.project.read().clone().ok_or("No project open")?;
    let board = state.board.read().clone();
    let schematics = state.schematics.read().clone();
    crate::io::native::save(&project, &board, &schematics, &path)?;
    let db = state.db.lock().map_err(|e| e.to_string())?;
    crate::db::project_store::save_project(&db, &project)?;
    store_last_project(&db, &path)?;
    *state.project.write() = Some(Project { path, ..project });
    Ok(())
}

#[tauri::command]
pub fn get_last_project(state: State<AppState>) -> Result<Option<String>, String> {
    let db = state.db.lock().map_err(|e| e.to_string())?;
    let path: Option<String> = db
        .query_row(
            "SELECT value FROM app_settings WHERE key = 'last_project_path'",
            [],
            |row| row.get(0),
        )
        .ok();
    drop(db);

    match path {
        Some(p) if std::path::Path::new(&p).exists() => Ok(Some(p)),
        _ => Ok(None),
    }
}

#[tauri::command]
pub fn close_project(state: State<AppState>) -> Result<(), String> {
    *state.project.write() = None;
    *state.board.write() = PcbBoard::default();
    state.schematics.write().clear();
    state.command_history.write().clear();
    Ok(())
}

#[tauri::command]
pub fn import_kicad_schematic(state: State<AppState>, path: String) -> Result<(), String> {
    let sheets = crate::io::kicad_sch::import_kicad_sch(&path)?;
    state.schematics.write().extend(sheets);
    Ok(())
}

#[tauri::command]
pub fn export_kicad_schematic(state: State<AppState>, path: String) -> Result<(), String> {
    let schematics = state.schematics.read();
    let sheet = schematics.values().next().ok_or("No schematic open")?;
    crate::io::kicad_sch::export_kicad_sch(sheet, &path)?;
    Ok(())
}

#[tauri::command]
pub fn import_kicad_pcb(state: State<AppState>, path: String) -> Result<(), String> {
    let board = crate::io::kicad_pcb::import_kicad_pcb(&path)?;
    *state.board.write() = board;
    Ok(())
}

#[tauri::command]
pub fn export_kicad_pcb(state: State<AppState>, path: String) -> Result<(), String> {
    let board = state.board.read().clone();
    crate::io::kicad_pcb::export_kicad_pcb(&board, &path)?;
    Ok(())
}

#[tauri::command]
pub fn export_gerber(state: State<AppState>, path: String) -> Result<(), String> {
    let board = state.board.read().clone();
    crate::io::gerber::export_gerber(&board, &path)?;
    Ok(())
}

#[tauri::command]
pub fn export_ipc2581(state: State<AppState>, path: String) -> Result<(), String> {
    let board = state.board.read().clone();
    crate::io::ipc2581::export_ipc2581(&board, &path)?;
    Ok(())
}

#[tauri::command]
pub fn export_bom(state: State<AppState>, path: String) -> Result<(), String> {
    let board = state.board.read().clone();
    crate::io::bom::generate_bom(&board, &path)?;
    Ok(())
}

#[tauri::command]
pub fn export_cpl(state: State<AppState>, path: String) -> Result<(), String> {
    let board = state.board.read().clone();
    crate::io::bom::generate_cpl(&board, &path)?;
    Ok(())
}

#[tauri::command]
pub fn export_spice_netlist(state: State<AppState>, path: String) -> Result<(), String> {
    let schematics = state.schematics.read();
    let sheet = schematics.values().next().ok_or("No schematic open")?;
    let params = std::collections::HashMap::new();
    crate::io::spice::export_spice(sheet, &path, &params)?;
    Ok(())
}

#[tauri::command]
pub fn run_spice_simulation(_state: State<AppState>, netlist_path: String, analysis_type: Option<String>) -> Result<String, String> {
    run_ngspice(&netlist_path, analysis_type.unwrap_or_else(|| "tran".to_string()))
}

fn run_ngspice(netlist_path: &str, analysis_type: String) -> Result<String, String> {
    let analysis = analysis_type.trim().to_ascii_lowercase();
    if !matches!(analysis.as_str(), "tran" | "ac" | "dc" | "op") {
        return Err(format!("Unsupported SPICE analysis type: {}", analysis_type));
    }

    // Check if ngspice is available
    let output = std::process::Command::new("ngspice")
        .args(["-b", "-r", "raw", netlist_path])
        .output()
        .map_err(|e| {
            if e.kind() == std::io::ErrorKind::NotFound {
                "ngspice not found. Install ngspice to run simulations. On Ubuntu: sudo apt install ngspice".to_string()
            } else {
                format!("Failed to execute ngspice: {}", e)
            }
        })?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(format!("ngspice failed: {}", stderr));
    }

    // Read the raw output file
    let raw_path = format!("{}.raw", netlist_path.strip_suffix(".cir").unwrap_or(netlist_path));
    let raw_data = std::fs::read_to_string(&raw_path)
        .unwrap_or_else(|_| String::from_utf8_lossy(&output.stdout).to_string());

    Ok(raw_data)
}
