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
pub async fn export_spice_netlist(state: State<'_, AppState>, path: String) -> Result<(), String> {
    let sheet = {
        let schematics = state.schematics.read();
        schematics.values().next().ok_or("No schematic open")?.clone()
    };
    let components = collect_components(&state, &sheet).await;
    let params = std::collections::HashMap::new();
    crate::io::spice::export_spice(&sheet, &path, &params, &components, None)
}

/// Run ngspice against the current schematic and return parsed plots.
#[tauri::command]
pub async fn simulate_schematic(
    state: State<'_, AppState>,
    analysis: Option<String>,
) -> Result<SpiceSimResult, String> {
    let sheet = {
        let schematics = state.schematics.read();
        schematics.values().next().ok_or("No schematic open")?.clone()
    };
    let components = collect_components(&state, &sheet).await;

    let params = std::collections::HashMap::new();
    let netlist = crate::io::spice::generate_spice(
        &sheet,
        &params,
        &components,
        analysis.as_deref(),
    )?;

    let dir = std::env::temp_dir().join("tracecraft-sim");
    std::fs::create_dir_all(&dir)
        .map_err(|e| format!("Could not create simulation temp directory: {}", e))?;
    let stamp = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_nanos())
        .unwrap_or_default();
    let netlist_path = dir.join(format!("sim-{}.cir", stamp));
    std::fs::write(&netlist_path, &netlist)
        .map_err(|e| format!("Could not write netlist: {}", e))?;

    let raw_path = dir.join(format!("sim-{}.raw", stamp));
    let (stdout, stderr) = run_ngspice(&netlist_path, &raw_path)?;

    let raw = std::fs::read_to_string(&raw_path).unwrap_or_default();
    let plots = parse_ascii_raw(&raw)?;
    if plots.is_empty() {
        let detail = if stderr.trim().is_empty() { stdout } else { stderr };
        return Err(format!(
            "ngspice produced no simulation data. Output:\n{}",
            detail.trim()
        ));
    }

    Ok(SpiceSimResult {
        plots,
        stdout,
        netlist,
    })
}

/// Run an existing netlist file and return parsed plots.
#[tauri::command]
pub fn run_spice_simulation(netlist_path: String) -> Result<SpiceSimResult, String> {
    let netlist = std::fs::read_to_string(&netlist_path)
        .map_err(|e| format!("Could not read netlist {}: {}", netlist_path, e))?;
    run_simulation_file(&netlist_path, &netlist)
}

fn run_simulation_file(netlist_path: &str, netlist: &str) -> Result<SpiceSimResult, String> {
    let dir = std::path::Path::new(netlist_path)
        .parent()
        .unwrap_or_else(|| std::path::Path::new("."));
    let stamp = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_nanos())
        .unwrap_or_default();
    let raw_path = dir.join(format!(".tracecraft-{}.raw", stamp));
    let (stdout, stderr) = run_ngspice(std::path::Path::new(netlist_path), &raw_path)?;
    let raw = std::fs::read_to_string(&raw_path).unwrap_or_default();
    let plots = parse_ascii_raw(&raw)?;
    if plots.is_empty() {
        let detail = if stderr.trim().is_empty() { stdout } else { stderr };
        return Err(format!(
            "ngspice produced no simulation data. Output:\n{}",
            detail.trim()
        ));
    }
    Ok(SpiceSimResult {
        plots,
        stdout,
        netlist: netlist.to_string(),
    })
}

pub(crate) async fn collect_components(
    state: &AppState,
    sheet: &SchematicSheet,
) -> crate::engine::netlist::ComponentMap {
    use std::collections::HashSet;
    let mut seen: HashSet<crate::model::library::ComponentId> = HashSet::new();
    let mut map = crate::engine::netlist::ComponentMap::new();
    for (_, sym) in sheet.symbols.iter() {
        if !seen.insert(sym.component_id) {
            continue;
        }
        match state.library.get_by_id(sym.component_id).await {
            Ok(Some(component)) => {
                map.insert(sym.component_id, component);
            }
            Ok(None) => {}
            Err(e) => {
                tracing::warn!(component = ?sym.component_id, error = %e, "failed to load component for simulation");
            }
        }
    }
    map
}

fn run_ngspice(
    netlist_path: &std::path::Path,
    raw_path: &std::path::Path,
) -> Result<(String, String), String> {
    let output = std::process::Command::new("ngspice")
        .arg("-b")
        .arg("-r")
        .arg(raw_path)
        .arg(netlist_path)
        .output()
        .map_err(|e| {
            if e.kind() == std::io::ErrorKind::NotFound {
                "ngspice not found. Install ngspice to run simulations. On Ubuntu: sudo apt install ngspice".to_string()
            } else {
                format!("Failed to execute ngspice: {}", e)
            }
        })?;

    let stdout = String::from_utf8_lossy(&output.stdout).to_string();
    let stderr = String::from_utf8_lossy(&output.stderr).to_string();

    if !output.status.success() {
        return Err(format!(
            "ngspice failed ({}):\n{}\n{}",
            output.status,
            stderr.trim(),
            stdout.trim()
        ));
    }

    Ok((stdout, stderr))
}

// ── ngspice ASCII rawfile parser ───────────────────────────────────────────

#[derive(Debug, Clone, serde::Serialize)]
pub struct SpicePlot {
    pub plot_name: String,
    pub variables: Vec<String>,
    pub points: usize,
    /// Column-major samples: `data[variable_index][point_index]`.
    pub data: Vec<Vec<f64>>,
}

#[derive(Debug, Clone, serde::Serialize)]
pub struct SpiceSimResult {
    pub plots: Vec<SpicePlot>,
    pub stdout: String,
    pub netlist: String,
}

fn parse_ascii_raw(raw: &str) -> Result<Vec<SpicePlot>, String> {
    let mut plots = Vec::new();
    let mut lines = raw.lines().peekable();

    while let Some(line) = lines.next() {
        let trimmed = line.trim();
        if !trimmed.starts_with("Plotname:") {
            continue;
        }
        let plot_name = trimmed
            .split_once(':')
            .map(|(_, v)| v.trim().to_string())
            .unwrap_or_default();

        let mut variables: Vec<String> = Vec::new();
        let mut declared_vars = 0usize;
        let mut points = 0usize;
        let mut complex = false;
        let mut in_variables = false;

        // Header block up to "Values:"
        let mut saw_values = false;
        while let Some(header_line) = lines.next() {
            let h = header_line.trim();
            if h.starts_with("No. Variables:") {
                declared_vars = h
                    .split_once(':')
                    .and_then(|(_, v)| v.trim().parse().ok())
                    .unwrap_or(0);
            } else if h.starts_with("No. Points:") {
                points = h
                    .split_once(':')
                    .and_then(|(_, v)| v.trim().parse().ok())
                    .unwrap_or(0);
            } else if h.starts_with("Flags:") && h.to_ascii_lowercase().contains("complex") {
                complex = true;
            } else if h == "Variables:" {
                in_variables = true;
            } else if h.starts_with("Values:") {
                saw_values = true;
                break;
            } else if in_variables && !h.is_empty() {
                let parts: Vec<&str> = h.split_whitespace().collect();
                if parts.len() >= 2 {
                    variables.push(parts[1].to_string());
                }
            }
        }

        if !saw_values {
            continue;
        }
        if complex {
            return Err("Complex rawfile plots are not supported".to_string());
        }
        if variables.is_empty() {
            variables = (0..declared_vars).map(|i| format!("v{}", i)).collect();
        }
        let nvars = declared_vars.max(variables.len());
        if nvars == 0 || points == 0 {
            continue;
        }

        // Collect numeric tokens; the first column of each point row is the
        // point index and is dropped.
        let mut values: Vec<f64> = Vec::new();
        for value_line in lines.by_ref() {
            let t = value_line.trim();
            if t.is_empty() {
                if values.len() >= nvars * points {
                    break;
                }
                continue;
            }
            if t.starts_with("Title:")
                || t.starts_with("Plotname:")
                || t.starts_with("Values:")
            {
                break;
            }
            let tokens: Vec<&str> = t.split_whitespace().collect();
            let mut start = 0;
            let at_point_boundary = values.len() % nvars == 0;
            if at_point_boundary {
                // The first line of each point row is "<index> <value>".
                if let Ok(index) = tokens[0].parse::<usize>() {
                    if index == values.len() / nvars {
                        start = 1;
                    }
                }
            } else if tokens.len() == 2 && value_line.starts_with('\t') {
                // Some writers prefix every variable line with its index.
                if tokens[0].parse::<usize>().ok() == Some(values.len() % nvars) {
                    start = 1;
                }
            }
            for token in &tokens[start..] {
                match token.parse::<f64>() {
                    Ok(v) => values.push(v),
                    Err(_) => {}
                }
            }
            if values.len() >= nvars * points {
                break;
            }
        }

        let available = values.len() / nvars;
        let point_count = points.min(available);
        if point_count == 0 {
            continue;
        }
        let mut data = vec![Vec::with_capacity(point_count); nvars];
        for point in 0..point_count {
            for var in 0..nvars {
                data[var].push(values[point * nvars + var]);
            }
        }
        variables.truncate(nvars);
        if variables.is_empty() {
            variables = (0..nvars).map(|i| format!("v{}", i)).collect();
        }

        plots.push(SpicePlot {
            plot_name,
            variables,
            points: point_count,
            data,
        });
    }

    Ok(plots)
}

#[cfg(test)]
mod raw_parser_tests {
    use super::parse_ascii_raw;

    const SAMPLE: &str = "Title: test\n\
Date: today\n\
Plotname: Transient Analysis\n\
Flags: real\n\
No. Variables: 3\n\
No. Points: 3\n\
Variables:\n\
\t0\ttime\ttime\n\
\t1\tv(out)\tvoltage\n\
\t2\ti(v1)\tcurrent\n\
Values:\n\
0\t0.000000000000000e+00\n\
\t0.000000000000000e+00\n\
\t0.000000000000000e+00\n\
1\t1.000000000000000e-03\n\
\t5.000000000000000e-01\n\
\t-1.000000000000000e-01\n\
2\t2.000000000000000e-03\n\
\t9.000000000000000e-01\n\
\t-2.000000000000000e-01\n";

    #[test]
    fn parses_plot_metadata_and_columns() {
        let plots = parse_ascii_raw(SAMPLE).expect("parse");
        assert_eq!(plots.len(), 1);
        let plot = &plots[0];
        assert_eq!(plot.plot_name, "Transient Analysis");
        assert_eq!(plot.variables, vec!["time", "v(out)", "i(v1)"]);
        assert_eq!(plot.points, 3);
        assert_eq!(plot.data[0], vec![0.0, 1e-3, 2e-3]);
        assert_eq!(plot.data[1], vec![0.0, 0.5, 0.9]);
        assert_eq!(plot.data[2], vec![0.0, -0.1, -0.2]);
    }

    #[test]
    fn rejects_complex_plots() {
        let text = SAMPLE.replace("Flags: real", "Flags: complex");
        assert!(parse_ascii_raw(&text).is_err());
    }

    #[test]
    fn tolerates_garbage_before_first_plot() {
        let text = format!("Some banner\n\n{}", SAMPLE);
        let plots = parse_ascii_raw(&text).unwrap();
        assert_eq!(plots.len(), 1);
    }
}
