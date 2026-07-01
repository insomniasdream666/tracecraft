use tauri::State;
use crate::app_state::AppState;
use crate::engine::drc;

#[tauri::command]
pub fn run_drc(
    state: State<AppState>,
) -> Result<Vec<drc::DRCViolation>, String> {
    let board = state.board.read();
    let violations = drc::run_drc_checks(&board);
    Ok(violations)
}

#[tauri::command]
pub fn get_drc_settings(
    state: State<AppState>,
) -> Result<crate::model::pcb::DesignRules, String> {
    let board = state.board.read();
    Ok(board.design_rules.clone())
}

#[tauri::command]
pub fn update_drc_settings(
    state: State<AppState>,
    rules: crate::model::pcb::DesignRules,
) -> Result<(), String> {
    let mut board = state.board.write();
    board.design_rules = rules;
    Ok(())
}
