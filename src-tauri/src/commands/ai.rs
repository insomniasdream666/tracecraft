use tauri::State;
use crate::app_state::AppState;
use crate::ai::chat::template_chat;
use crate::ai::recommender::keyword_recommend;
use crate::model::geometry::Point;
use crate::model::id::NetId;

#[tauri::command]
pub fn ai_chat(
    state: State<AppState>,
    message: String,
) -> Result<String, String> {
    let board = state.board.read();
    let schematics = state.schematics.read();
    let schematic = schematics.values().next();
    let board_ref = Some(&*board);
    template_chat(&message, board_ref, schematic.map(|s| s as &crate::model::schematic::SchematicSheet))
}

#[tauri::command]
pub fn recommend_components(
    _state: State<AppState>,
    context: String,
) -> Result<Vec<String>, String> {
    let recs = keyword_recommend(&context);
    Ok(recs.into_iter().map(|r| format!("{} - {} (score: {:.1})", r.name, r.description, r.score)).collect())
}

#[tauri::command]
pub fn suggest_route(
    state: State<AppState>,
    _net_id: String,
    from_x: f64,
    from_y: f64,
    to_x: f64,
    to_y: f64,
) -> Result<Vec<Vec<f64>>, String> {
    let board = state.board.read();
    let from = Point { x: from_x, y: from_y };
    let to = Point { x: to_x, y: to_y };
    let suggestions = crate::ai::routing_assist::suggest_route(&board, NetId(0), from, to);
    Ok(suggestions.into_iter().map(|s| s.points.into_iter().flat_map(|p| vec![p.x, p.y]).collect()).collect())
}
