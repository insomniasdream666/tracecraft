use crate::model::library::*;
use std::collections::HashMap;

/// Import from a KiCad sym-lib file.
pub fn import_kicad_sym_lib(path: &str) -> Result<Vec<Component>, String> {
    let text = std::fs::read_to_string(path).map_err(|e| e.to_string())?;
    let parsed = super::kicad_sch::parse_sexpr(&text)?;
    let root = super::kicad_sch::Sexpr::List(parsed);
    let mut components = Vec::new();

    let defs = super::kicad_sch::find_all(&root, "symbol");
    for sym_expr in defs {
        let name = super::kicad_sch::get_str(sym_expr, 1).unwrap_or("Unknown");
        let mut pins = Vec::new();
        let mut body = Vec::new();

        for pin_expr in super::kicad_sch::find_all(sym_expr, "pin") {
            let pin_name = super::kicad_sch::find_first_child(pin_expr, "name").unwrap_or_default();
            let pin_num = super::kicad_sch::find_first_child(pin_expr, "number").unwrap_or_default();
            let pos = super::kicad_sch::find_at(pin_expr).unwrap_or((0.0, 0.0));
            let el_type = match super::kicad_sch::get_str(pin_expr, 1).unwrap_or("") {
                "input" => PinType::Input,
                "output" => PinType::Output,
                "bidirectional" => PinType::Bidirectional,
                "passive" => PinType::Passive,
                "power_in" => PinType::Power,
                "power_out" => PinType::Power,
                "open_collector" => PinType::OpenCollector,
                "open_emitter" => PinType::OpenEmitter,
                "not_connected" => PinType::NotConnected,
                _ => PinType::Passive,
            };
            pins.push(PinDef {
                name: pin_name,
                number: pin_num,
                position: crate::model::geometry::Point { x: pos.0, y: pos.1 },
                length: 2.54,
                electrical_type: el_type,
            });
        }

        for rect_expr in super::kicad_sch::find_all(sym_expr, "rectangle") {
            let start = super::kicad_sch::find_attr(rect_expr, "start").unwrap_or((0.0, 0.0));
            let end = super::kicad_sch::find_attr(rect_expr, "end").unwrap_or((0.0, 0.0));
            let w = (end.0 - start.0).abs();
            let h = (end.1 - start.1).abs();
            let x = start.0.min(end.0);
            let y = start.1.min(end.1);
            body.push(SymbolGraphics::Rectangle {
                position: crate::model::geometry::Point { x, y },
                size: crate::model::geometry::Size { width: w, height: h },
                fill: "none".into(),
                stroke: "#000".into(),
            });
        }

        let category = guess_category(name);
        components.push(Component {
            id: ComponentId(uuid::Uuid::new_v4()),
            name: name.to_string(),
            description: format!("Imported from {}", path),
            manufacturer: None,
            category: Some(category),
            symbol: SymbolDef { pins, body, origin: crate::model::geometry::Point::new(0.0, 0.0) },
            footprint: FootprintDef::default(),
            model_3d: None,
            parameters: HashMap::new(),
            created_at: chrono::Utc::now().to_rfc3339(),
            updated_at: chrono::Utc::now().to_rfc3339(),
        });
    }

    Ok(components)
}

/// Import from a CSV BOM (e.g. DigiKey, Mouser export).
/// Expects columns: Reference, Value, Manufacturer, PartNumber, Category
pub fn import_csv_bom(path: &str) -> Result<Vec<Component>, String> {
    let mut rdr = csv::Reader::from_path(path).map_err(|e| e.to_string())?;
    let mut components = Vec::new();

    for result in rdr.records() {
        let record = result.map_err(|e| e.to_string())?;
        let name = record.get(1).unwrap_or("").to_string();
        let manufacturer = record.get(2).map(|s| s.to_string());
        let part_number = record.get(3).unwrap_or("").to_string();
        let category = record.get(4).unwrap_or("misc").to_string();

        if name.is_empty() { continue; }

        components.push(Component {
            id: ComponentId(uuid::Uuid::new_v4()),
            name: if part_number.is_empty() { name.clone() } else { format!("{} ({})", name, part_number) },
            description: format!("Imported from {}", path),
            manufacturer,
            category: Some(category),
            symbol: SymbolDef::default(),
            footprint: FootprintDef::default(),
            model_3d: None,
            parameters: {
                let mut m = HashMap::new();
                m.insert("part_number".into(), part_number);
                m
            },
            created_at: chrono::Utc::now().to_rfc3339(),
            updated_at: chrono::Utc::now().to_rfc3339(),
        });
    }

    Ok(components)
}

/// Import from a KiCad .kicad_mod footprint file or .pretty directory.
pub fn import_kicad_pretty_lib(path: &str) -> Result<Vec<Component>, String> {
    let path_meta = std::fs::metadata(path).map_err(|e| e.to_string())?;

    let entries: Vec<(String, FootprintDef, Option<Model3D>)> = if path_meta.is_dir() {
        super::kicad_pcb::import_kicad_pretty_directory(path)?
    } else {
        let (name, fp_def, model) = super::kicad_pcb::parse_kicad_mod(path)?;
        vec![(name, fp_def, model)]
    };

    let mut components = Vec::new();
    for (name, fp_def, model_3d) in entries {
        let category = guess_category(&name);
        components.push(Component {
            id: ComponentId(uuid::Uuid::new_v4()),
            name,
            description: format!("Imported footprint from {}", path),
            manufacturer: None,
            category: Some(category),
            symbol: SymbolDef::default(),
            footprint: fp_def,
            model_3d,
            parameters: std::collections::HashMap::new(),
            created_at: chrono::Utc::now().to_rfc3339(),
            updated_at: chrono::Utc::now().to_rfc3339(),
        });
    }

    Ok(components)
}

fn guess_category(name: &str) -> String {
    let lower = name.to_lowercase();
    if lower.contains("resistor") || lower.contains('r') && lower.len() < 5 { "resistor".into() }
    else if lower.contains("capacitor") || lower.contains('c') && lower.len() < 5 { "capacitor".into() }
    else if lower.contains("inductor") || lower.contains('l') && lower.len() < 5 { "inductor".into() }
    else if lower.contains("diode") || lower.contains("led") { "diode".into() }
    else if lower.contains("transistor") || lower.contains("mosfet") { "transistor".into() }
    else if lower.contains("ic") || lower.contains("chip") || lower.contains("mcu") { "ic".into() }
    else if lower.contains("connector") || lower.contains("header") || lower.contains("jack") { "connector".into() }
    else if lower.contains("regulator") || lower.contains("ldo") || lower.contains("dc") { "power".into() }
    else if lower.contains("sensor") || lower.contains("temp") { "sensor".into() }
    else if lower.contains("relay") || lower.contains("switch") || lower.contains("buzzer") { "electromechanical".into() }
    else { "misc".into() }
}
