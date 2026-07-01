use tauri::State;
use crate::app_state::AppState;
use crate::model::project::*;
use crate::model::pcb::PcbBoard;

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
    let db = state.db.lock().map_err(|e| e.to_string())?;
    let id = uuid::Uuid::new_v4();
    let project = crate::db::project_store::load_project(&db, id)?
        .unwrap_or_else(|| Project {
            id,
            name: path.split('/').last().unwrap_or("project").to_string(),
            path: path.clone(),
            created_at: chrono::Utc::now().to_rfc3339(),
            updated_at: chrono::Utc::now().to_rfc3339(),
        });
    let summary = ProjectSummary {
        id: project.id,
        name: project.name.clone(),
        path: project.path.clone(),
        updated_at: project.updated_at.clone(),
    };
    *state.project.write() = Some(project);
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
    *state.project.write() = Some(Project { path, ..project });
    Ok(())
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
