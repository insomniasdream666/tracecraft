use std::collections::HashMap;
use tauri::State;
use crate::app_state::AppState;
use crate::engine::erc::{run_erc, ERCViolation};
use crate::model::schematic::*;
use crate::model::id::*;
use crate::undo::command::*;
use crate::model::geometry::*;
use crate::model::schematic::*;

#[tauri::command]
pub fn get_schematic(state: State<AppState>, _sheet_id: Option<SheetId>) -> Result<Option<SchematicSheet>, String> {
    let schematics = state.schematics.read();
    let sheet = schematics.values().next().cloned();
    Ok(sheet)
}

#[tauri::command]
pub fn place_symbol(
    state: State<AppState>,
    sheet_id: SheetId,
    component_id: crate::model::library::ComponentId,
    position: Point,
) -> Result<SymbolId, String> {
    let mut schematics = state.schematics.write();
    let sheet = schematics.get_mut(&sheet_id).ok_or("Sheet not found")?;
    let sym = SymbolInstance {
        component_id,
        position,
        rotation: Rotation(0.0),
        mirrored: false,
        refdes: String::new(),
        value: String::new(),
    };
    let id = SymbolId(sheet.symbols.insert(sym));
    Ok(id)
}

#[tauri::command]
pub fn draw_wire(
    state: State<AppState>,
    sheet_id: SheetId,
    points: Vec<Point>,
    width: Option<f64>,
) -> Result<WireId, String> {
    let mut schematics = state.schematics.write();
    let sheet = schematics.get_mut(&sheet_id).ok_or("Sheet not found")?;
    let wire = Wire {
        points,
        net_id: None,
        width: width.unwrap_or(0.25),
    };
    let id = WireId(sheet.wires.insert(wire));
    Ok(id)
}

#[tauri::command]
pub fn delete_element(
    state: State<AppState>,
    sheet_id: SheetId,
    element_type: String,
    element_id: usize,
) -> Result<(), String> {
    let mut schematics = state.schematics.write();
    let sheet = schematics.get_mut(&sheet_id).ok_or("Sheet not found")?;
    match element_type.as_str() {
        "symbol" => { sheet.symbols.remove(element_id); }
        "wire" => { sheet.wires.remove(element_id); }
        "junction" => { sheet.junctions.remove(element_id); }
        "label" => { sheet.net_labels.remove(element_id); }
        _ => return Err(format!("Unknown element type: {}", element_type)),
    }
    Ok(())
}

#[tauri::command]
pub fn add_net_label(
    state: State<AppState>,
    sheet_id: SheetId,
    name: String,
    position: Point,
) -> Result<NetLabelId, String> {
    let mut schematics = state.schematics.write();
    let sheet = schematics.get_mut(&sheet_id).ok_or("Sheet not found")?;
    let label = NetLabel { name, position };
    let id = NetLabelId(sheet.net_labels.insert(label));
    Ok(id)
}

#[tauri::command]
pub fn add_junction(
    state: State<AppState>,
    sheet_id: SheetId,
    position: Point,
) -> Result<JunctionId, String> {
    let mut schematics = state.schematics.write();
    let sheet = schematics.get_mut(&sheet_id).ok_or("Sheet not found")?;
    let junction = Junction { position };
    let id = JunctionId(sheet.junctions.insert(junction));
    Ok(id)
}

#[tauri::command]
pub fn run_erc_command(state: State<AppState>) -> Result<Vec<ERCViolation>, String> {
    let schematics = state.schematics.read();
    let sheet = schematics.values().next().ok_or("No schematic sheet found")?;
    let violations = run_erc(sheet);
    Ok(violations)
}

#[tauri::command]
pub fn annotate_schematic(state: State<AppState>, sheet_id: SheetId) -> Result<(), String> {
    let mut schematics = state.schematics.write();
    let sheet = schematics.get_mut(&sheet_id).ok_or("Sheet not found")?;

    // Count existing refdes prefixes
    let mut counters: HashMap<String, usize> = HashMap::new();

    // Build a list of symbol IDs sorted by position (top-left first)
    let mut symbol_ids: Vec<usize> = sheet.symbols.iter().map(|(id, _)| id).collect();
    symbol_ids.sort_by(|&a, &b| {
        let pa = sheet.symbols.get(a).map(|s| s.position).unwrap_or(Point::new(0.0, 0.0));
        let pb = sheet.symbols.get(b).map(|s| s.position).unwrap_or(Point::new(0.0, 0.0));
        (pa.y * 1000.0 + pa.x).partial_cmp(&(pb.y * 1000.0 + pb.x)).unwrap_or(std::cmp::Ordering::Equal)
    });

    // Refdes prefix mapping by component type heuristic
    fn prefix_for(name: &str) -> &str {
        let lower = name.to_lowercase();
        if lower.contains("resistor") || lower.starts_with('r') { "R" }
        else if lower.contains("capacitor") || lower.starts_with('c') { "C" }
        else if lower.contains("inductor") || lower.starts_with('l') { "L" }
        else if lower.contains("diode") || lower.starts_with('d') { "D" }
        else if lower.contains("transistor") || lower.starts_with('q') { "Q" }
        else if lower.contains("connector") || lower.starts_with('j') || lower.starts_with('p') { "J" }
        else if lower.contains("ic") || lower.contains("integrated") || lower.starts_with('u') { "U" }
        else { "U" }
    }

    for sym_id in symbol_ids {
        if let Some(sym) = sheet.symbols.get_mut(sym_id) {
            if sym.refdes.is_empty() {
                let prefix = prefix_for(&sym.value);
                let count = counters.entry(prefix.to_string()).or_insert(0);
                *count += 1;
                sym.refdes = format!("{}{}", prefix, count);
            }
        }
    }

    Ok(())
}

#[tauri::command]
pub fn create_bus(
    state: State<AppState>,
    sheet_id: SheetId,
    name: String,
    members: Vec<String>,
) -> Result<BusId, String> {
    let mut schematics = state.schematics.write();
    let sheet = schematics.get_mut(&sheet_id).ok_or("Sheet not found")?;
    let bus = Bus { name, members, wires: vec![] };
    let id = BusId(sheet.buses.insert(bus));
    Ok(id)
}

#[tauri::command]
pub fn create_hierarchical_sheet(
    state: State<AppState>,
    sheet_id: SheetId,
    name: String,
    position: Point,
) -> Result<SheetId, String> {
    let mut schematics = state.schematics.write();
    let child_id = SheetId(schematics.len());

    let child = SchematicSheet {
        id: child_id,
        name: name.clone(),
        symbols: Default::default(),
        wires: Default::default(),
        junctions: Default::default(),
        net_labels: Default::default(),
        buses: Default::default(),
        text_annotations: Default::default(),
        hier_sheets: Default::default(),
        properties: HashMap::new(),
    };
    schematics.insert(child_id, child);

    if let Some(sheet) = schematics.get_mut(&sheet_id) {
        let hier = HierarchicalSheet {
            id: SheetId(sheet.hier_sheets.len()),
            name,
            child_sheet_id: child_id,
            pins: vec![],
            position,
            size: Size { width: 200.0, height: 150.0 },
        };
        sheet.hier_sheets.insert(hier);
    }

    Ok(child_id)
}

#[tauri::command]
pub fn move_elements(
    state: State<AppState>,
    sheet_id: SheetId,
    moves: Vec<ElementMove>,
) -> Result<(), String> {
    let mut schematics = state.schematics.write();
    let sheet = schematics.get_mut(&sheet_id).ok_or("Sheet not found")?;
    for m in moves {
        match m.element_type.as_str() {
            "symbol" => {
                if let Some(sym) = sheet.symbols.get_mut(m.element_id) {
                    sym.position = m.new_position;
                }
            }
            "wire" => {
                // For wires, shift all points by delta
                if let Some(wire) = sheet.wires.get_mut(m.element_id) {
                    let dx = m.new_position.x - m.old_position.x;
                    let dy = m.new_position.y - m.old_position.y;
                    for pt in &mut wire.points {
                        pt.x += dx;
                        pt.y += dy;
                    }
                }
            }
            _ => {}
        }
    }
    Ok(())
}

#[derive(serde::Deserialize)]
pub struct ElementMove {
    pub element_type: String,
    pub element_id: usize,
    pub old_position: Point,
    pub new_position: Point,
}

#[tauri::command]
pub fn undo_schematic(state: State<AppState>) -> Result<(), String> {
    let mut board = state.board.write();
    let mut schematics = state.schematics.write();
    let mut history = state.command_history.write();
    if let Some(sheet) = schematics.values_mut().next() {
        history.undo(&mut board, sheet)
    } else {
        let mut dummy = crate::model::schematic::SchematicSheet::default();
        history.undo(&mut board, &mut dummy)
    }
}

#[tauri::command]
pub fn redo_schematic(state: State<AppState>) -> Result<(), String> {
    let mut board = state.board.write();
    let mut schematics = state.schematics.write();
    let mut history = state.command_history.write();
    if let Some(sheet) = schematics.values_mut().next() {
        history.redo(&mut board, sheet)
    } else {
        let mut dummy = crate::model::schematic::SchematicSheet::default();
        history.redo(&mut board, &mut dummy)
    }
}
