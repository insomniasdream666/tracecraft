use rusqlite::Connection;
use crate::model::library::*;
use crate::model::id::{ComponentId, LayerId};
use crate::model::geometry::*;
use std::collections::HashMap;

const SEED_COMPONENTS: &[SeedComponent] = &[
    SeedComponent {
        name: "Resistor",
        description: "Generic resistor",
        category: "resistor",
        parameters: &[("Resistance", "1k"), ("Power Rating", "0.25W"), ("Tolerance", "5%")],
        pins: &[("1", "Passive", -2.54, 0.0), ("2", "Passive", 2.54, 0.0)],
        body: &[
            ("line", -1.27, 0.0, -0.42, 0.0, "", "#E8E9F3", 0.3),
            ("line", -0.42, 0.0, -0.42, -1.27, "", "#E8E9F3", 0.3),
            ("line", -0.42, -1.27, 0.42, 1.27, "", "#E8E9F3", 0.3),
            ("line", 0.42, 1.27, 0.42, -1.27, "", "#E8E9F3", 0.3),
            ("line", 0.42, -1.27, 1.27, 0.0, "", "#E8E9F3", 0.3),
        ],
        pads: &[("1", "Round", -2.54, 0.0, 0.6, 0.6, None), ("2", "Round", 2.54, 0.0, 0.6, 0.6, None)],
        outline: &[(-3.81, -1.27), (3.81, -1.27), (3.81, 1.27), (-3.81, 1.27)],
    },
    SeedComponent {
        name: "Capacitor",
        description: "Generic ceramic capacitor",
        category: "capacitor",
        parameters: &[("Capacitance", "100nF"), ("Voltage Rating", "50V"), ("Dielectric", "X7R")],
        pins: &[("1", "Passive", -2.54, 0.0), ("2", "Passive", 2.54, 0.0)],
        body: &[
            ("line", -0.51, -1.27, -0.51, 1.27, "", "#E8E9F3", 0.3),
            ("line", 0.51, -1.27, 0.51, 1.27, "", "#E8E9F3", 0.3),
            ("line", -2.54, 0.0, -0.51, 0.0, "", "#E8E9F3", 0.3),
            ("line", 0.51, 0.0, 2.54, 0.0, "", "#E8E9F3", 0.3),
        ],
        pads: &[("1", "Round", -2.54, 0.0, 0.6, 0.6, None), ("2", "Round", 2.54, 0.0, 0.6, 0.6, None)],
        outline: &[(-3.81, -1.52), (3.81, -1.52), (3.81, 1.52), (-3.81, 1.52)],
    },
    SeedComponent {
        name: "Diode",
        description: "Generic signal diode",
        category: "diode",
        parameters: &[("Type", "Signal"), ("Vr", "100V"), ("If", "200mA")],
        pins: &[("A", "Passive", -2.54, 0.0), ("K", "Passive", 2.54, 0.0)],
        body: &[
            ("line", -2.54, 0.0, -0.51, 0.0, "", "#E8E9F3", 0.3),
            ("line", 0.51, 0.0, 2.54, 0.0, "", "#E8E9F3", 0.3),
            ("line", 0.0, -1.27, 0.0, 1.27, "", "#E8E9F3", 0.3),
            ("line", 0.0, -1.27, 1.27, 0.0, "", "#E8E9F3", 0.3),
            ("line", 1.27, 0.0, 0.0, 1.27, "", "#E8E9F3", 0.3),
        ],
        pads: &[("1", "Round", -2.54, 0.0, 0.6, 0.6, None), ("2", "Round", 2.54, 0.0, 0.6, 0.6, None)],
        outline: &[(-3.81, -1.52), (3.81, -1.52), (3.81, 1.52), (-3.81, 1.52)],
    },
    SeedComponent {
        name: "LED",
        description: "Generic LED",
        category: "diode",
        parameters: &[("Color", "Red"), ("If", "20mA"), ("Vf", "2.0V")],
        pins: &[("A", "Passive", -2.54, 0.0), ("K", "Passive", 2.54, 0.0)],
        body: &[
            ("line", -2.54, 0.0, -0.51, 0.0, "", "#E8E9F3", 0.3),
            ("line", 0.51, 0.0, 2.54, 0.0, "", "#E8E9F3", 0.3),
            ("line", 0.0, -1.27, 0.0, 1.27, "", "#E8E9F3", 0.3),
            ("line", 0.0, -1.27, 1.27, 0.0, "", "#E8E9F3", 0.3),
            ("line", 1.27, 0.0, 0.0, 1.27, "", "#E8E9F3", 0.3),
            ("line", 1.52, -2.03, 1.78, -1.52, "", "#E8E9F3", 0.3),
            ("line", 1.78, -1.52, 2.29, -1.78, "", "#E8E9F3", 0.3),
            ("line", 2.29, -2.29, 2.54, -1.78, "", "#E8E9F3", 0.3),
            ("text", -1.0, 1.78, 0.5, 0.0, "LED", "", 0.0),
        ],
        pads: &[("1", "Round", -2.54, 0.0, 0.6, 0.6, None), ("2", "Round", 2.54, 0.0, 0.6, 0.6, None)],
        outline: &[(-3.81, -2.54), (3.81, -2.54), (3.81, 2.54), (-3.81, 2.54)],
    },
    SeedComponent {
        name: "NPN Transistor",
        description: "Generic NPN BJT transistor",
        category: "transistor",
        parameters: &[("Type", "NPN"), ("Vceo", "40V"), ("Ic", "200mA"), ("hFE", "100")],
        pins: &[("B", "Input", -1.27, 1.27), ("C", "Output", 2.54, -1.27), ("E", "Output", 2.54, 1.27)],
        body: &[
            ("line", -1.27, 1.27, 0.0, 0.0, "", "#E8E9F3", 0.3),
            ("line", 0.0, 0.0, 0.0, -1.27, "", "#E8E9F3", 0.3),
            ("line", 0.0, -1.27, 2.54, -1.27, "", "#E8E9F3", 0.3),
            ("line", 0.0, 0.0, 0.0, 1.27, "", "#E8E9F3", 0.3),
            ("line", 0.0, 1.27, 2.54, 1.27, "", "#E8E9F3", 0.3),
            ("line", 2.54, -1.27, 2.54, 1.27, "", "#E8E9F3", 0.3),
            ("line", 1.02, -0.25, 1.78, -0.76, "", "#E8E9F3", 0.3),
            ("line", 1.02, 0.25, 1.78, 0.76, "", "#E8E9F3", 0.3),
        ],
        pads: &[
            ("1", "Round", -1.27, 1.27, 0.6, 0.6, None),
            ("2", "Round", 2.54, -1.27, 0.6, 0.6, None),
            ("3", "Round", 2.54, 1.27, 0.6, 0.6, None),
        ],
        outline: &[(-2.54, -1.52), (3.81, -1.52), (3.81, 1.52), (-2.54, 1.52)],
    },
    SeedComponent {
        name: "GND",
        description: "Ground power symbol",
        category: "power",
        parameters: &[],
        pins: &[("1", "Gnd", 0.0, 0.0)],
        body: &[
            ("line", 0.0, 0.0, 0.0, 0.51, "", "#E8E9F3", 0.3),
            ("line", -1.27, 0.51, 1.27, 0.51, "", "#E8E9F3", 0.3),
            ("line", -0.76, 1.02, 0.76, 1.02, "", "#E8E9F3", 0.3),
            ("line", -0.25, 1.52, 0.25, 1.52, "", "#E8E9F3", 0.3),
        ],
        pads: &[("1", "Round", 0.0, 0.0, 0.6, 0.6, None)],
        outline: &[(-1.52, 0.0), (1.52, 0.0), (1.52, 1.78), (-1.52, 1.78)],
    },
    SeedComponent {
        name: "VCC",
        description: "Power supply symbol",
        category: "power",
        parameters: &[("Voltage", "5V")],
        pins: &[("1", "Power", 0.0, 0.0)],
        body: &[
            ("line", 0.0, 0.0, 0.0, -1.27, "", "#E8E9F3", 0.3),
            ("line", -1.27, -1.27, 1.27, -1.27, "", "#E8E9F3", 0.3),
            ("line", 0.0, -1.27, 0.0, -2.03, "", "#E8E9F3", 0.3),
            ("text", -0.64, -2.79, 0.5, 0.0, "VCC", "", 0.0),
        ],
        pads: &[("1", "Round", 0.0, 0.0, 0.6, 0.6, None)],
        outline: &[(-1.52, -3.05), (1.52, -3.05), (1.52, 0.0), (-1.52, 0.0)],
    },
];

struct SeedComponent {
    name: &'static str,
    description: &'static str,
    category: &'static str,
    parameters: &'static [(&'static str, &'static str)],
    pins: &'static [(&'static str, &'static str, f64, f64)],
    body: &'static [(&'static str, f64, f64, f64, f64, &'static str, &'static str, f64)],
    pads: &'static [(&'static str, &'static str, f64, f64, f64, f64, Option<f64>)],
    outline: &'static [(f64, f64)],
}

fn parse_electrical_type(s: &str) -> PinType {
    match s {
        "Input" => PinType::Input,
        "Output" => PinType::Output,
        "Bidirectional" => PinType::Bidirectional,
        "Power" => PinType::Power,
        "Gnd" => PinType::Gnd,
        "Passive" => PinType::Passive,
        "OpenCollector" => PinType::OpenCollector,
        "OpenEmitter" => PinType::OpenEmitter,
        "NotConnected" => PinType::NotConnected,
        _ => PinType::Passive,
    }
}

fn parse_pad_shape(s: &str) -> PadShape {
    match s {
        "Round" => PadShape::Round,
        "Rect" => PadShape::Rect,
        "Oval" => PadShape::Oval,
        "RoundRect" => PadShape::RoundRect,
        _ => PadShape::Round,
    }
}

fn make_component(seed: &SeedComponent) -> Component {
    let id = ComponentId(uuid::Uuid::new_v4());
    let now = chrono::Utc::now().to_rfc3339();

    let pins: Vec<PinDef> = seed.pins.iter().map(|(num, etype, x, y)| {
        PinDef {
            name: num.to_string(),
            number: num.to_string(),
            position: Point::new(*x, *y),
            length: 2.54,
            electrical_type: parse_electrical_type(etype),
        }
    }).collect();

    let body: Vec<SymbolGraphics> = seed.body.iter().map(|(kind, x1, y1, x2, y2, fill, stroke, width)| {
        match *kind {
            "line" => SymbolGraphics::Line {
                start: Point::new(*x1, *y1),
                end: Point::new(*x2, *y2),
                stroke: stroke.to_string(),
                width: *width,
            },
            "text" => SymbolGraphics::Text {
                text: fill.to_string(),
                position: Point::new(*x1, *y1),
                size: *x2,
            },
            _ => SymbolGraphics::Line {
                start: Point::new(0.0, 0.0),
                end: Point::new(0.0, 0.0),
                stroke: "#E8E9F3".to_string(),
                width: 0.3,
            },
        }
    }).collect();

    let pads: Vec<PadDef> = seed.pads.iter().map(|(num, shape, x, y, w, h, drill)| {
        PadDef {
            number: num.to_string(),
            shape: parse_pad_shape(shape),
            position: Point::new(*x, *y),
            size: Size { width: *w, height: *h },
            layer: LayerId(1),
            drill: *drill,
        }
    }).collect();

    let outline: Vec<Point> = seed.outline.iter().map(|(x, y)| Point::new(*x, *y)).collect();

    let mut parameters = HashMap::new();
    for (k, v) in seed.parameters {
        parameters.insert(k.to_string(), v.to_string());
    }

    Component {
        id,
        name: seed.name.to_string(),
        description: seed.description.to_string(),
        manufacturer: None,
        category: Some(seed.category.to_string()),
        symbol: SymbolDef { pins, body, origin: Point::new(0.0, 0.0) },
        footprint: FootprintDef { pads, outline, courtyard: vec![] },
        model_3d: None,
        parameters,
        created_at: now.clone(),
        updated_at: now,
    }
}

pub fn seed_components(conn: &Connection) -> Result<(), String> {
    let count: i64 = conn
        .query_row("SELECT COUNT(*) FROM components", [], |r| r.get(0))
        .map_err(|e| e.to_string())?;

    if count > 0 {
        return Ok(());
    }

    for seed in SEED_COMPONENTS {
        let component = make_component(seed);
        let symbol_json = serde_json::to_string(&component.symbol).map_err(|e| e.to_string())?;
        let footprint_json = serde_json::to_string(&component.footprint).map_err(|e| e.to_string())?;
        let parameters_json = serde_json::to_string(&component.parameters).map_err(|e| e.to_string())?;
        let model_3d_json: Option<String> = None;

        conn.execute(
            "INSERT INTO components (id, name, description, manufacturer, category,
             symbol_json, footprint_json, model_3d_json, parameters_json, created_at, updated_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11)",
            rusqlite::params![
                component.id.0.to_string(),
                component.name,
                component.description,
                component.manufacturer,
                component.category,
                symbol_json,
                footprint_json,
                model_3d_json,
                parameters_json,
                component.created_at,
                component.updated_at,
            ],
        ).map_err(|e| e.to_string())?;
    }

    Ok(())
}
