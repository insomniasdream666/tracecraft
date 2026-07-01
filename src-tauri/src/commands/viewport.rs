use tauri::State;
use crate::app_state::AppState;
use crate::model::geometry::{Point, Rect};
use crate::model::id::SheetId;

#[tauri::command]
pub fn get_board_bounds(state: State<AppState>) -> Result<Rect, String> {
    let board = state.board.read();
    let mut points: Vec<Point> = Vec::new();

    for (_, fp) in board.footprints.iter() {
        points.push(fp.position);
    }
    for (_, trace) in board.traces.iter() {
        points.extend(trace.points.iter().copied());
    }
    for (_, via) in board.vias.iter() {
        points.push(via.position);
    }
    for (_, zone) in board.zones.iter() {
        points.extend(zone.outline.iter().copied());
    }
    if let Some(ref outline) = board.outline {
        for contour in &outline.shape {
            points.extend(contour.iter().copied());
        }
        for cutout in &outline.cutouts {
            points.extend(cutout.iter().copied());
        }
    }

    Rect::from_points(&points).ok_or_else(|| "Board is empty".to_string())
}

#[tauri::command]
pub fn get_schematic_bounds(
    state: State<AppState>,
    sheet_id: SheetId,
) -> Result<Rect, String> {
    let schematics = state.schematics.read();
    let sheet = schematics
        .get(&sheet_id)
        .ok_or_else(|| "Sheet not found".to_string())?;

    let mut points: Vec<Point> = Vec::new();

    for (_, sym) in sheet.symbols.iter() {
        points.push(sym.position);
    }
    for (_, wire) in sheet.wires.iter() {
        points.extend(wire.points.iter().copied());
    }
    for (_, junc) in sheet.junctions.iter() {
        points.push(junc.position);
    }
    for (_, lbl) in sheet.net_labels.iter() {
        points.push(lbl.position);
    }
    for (_, txt) in sheet.text_annotations.iter() {
        points.push(txt.position);
    }
    for (_, hs) in sheet.hier_sheets.iter() {
        points.push(hs.position);
        points.push(Point::new(
            hs.position.x + hs.size.width,
            hs.position.y + hs.size.height,
        ));
    }

    Rect::from_points(&points).ok_or_else(|| "Sheet is empty".to_string())
}
