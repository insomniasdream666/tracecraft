use std::collections::HashMap;
use crate::model::id::{SheetId, ComponentId};
use crate::model::geometry::{Point, Rotation, Size};
use crate::model::arena::Arena;
use crate::model::schematic::*;

#[derive(Debug, Clone)]
pub enum Sexpr {
    Str(String),
    Int(i64),
    Float(f64),
    List(Vec<Sexpr>),
}

impl std::fmt::Display for Sexpr {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Sexpr::Str(s) => write!(f, "{}", escape_sexpr(s)),
            Sexpr::Int(n) => write!(f, "{}", n),
            Sexpr::Float(n) => write!(f, "{:.6}", n),
            Sexpr::List(items) => {
                write!(f, "(")?;
                for (i, item) in items.iter().enumerate() {
                    if i > 0 { write!(f, " ")?; }
                    write!(f, "{}", item)?;
                }
                write!(f, ")")
            }
        }
    }
}

pub fn escape_sexpr(s: &str) -> String {
    if s.contains(' ') || s.contains('"') || s.contains('(') || s.contains(')') {
        format!("\"{}\"", s.replace('\\', "\\\\").replace('"', "\\\""))
    } else {
        s.to_string()
    }
}

pub fn parse_sexpr(input: &str) -> Result<Vec<Sexpr>, String> {
    let chars = input.chars().collect::<Vec<_>>();
    let mut pos = 0;
    let mut results = Vec::new();
    while pos < chars.len() {
        while pos < chars.len() && chars[pos].is_whitespace() { pos += 1; }
        if pos >= chars.len() { break; }
        results.push(parse_one(&chars, &mut pos)?);
    }
    Ok(results)
}

fn parse_one(chars: &[char], pos: &mut usize) -> Result<Sexpr, String> {
    while *pos < chars.len() && chars[*pos].is_whitespace() { *pos += 1; }
    if *pos >= chars.len() { return Err("Unexpected end".to_string()); }
    if chars[*pos] == '(' {
        *pos += 1;
        let mut items = Vec::new();
        while *pos < chars.len() {
            while *pos < chars.len() && chars[*pos].is_whitespace() { *pos += 1; }
            if *pos >= chars.len() { break; }
            if chars[*pos] == ')' { *pos += 1; return Ok(Sexpr::List(items)); }
            items.push(parse_one(chars, pos)?);
        }
        Err("Unmatched paren".to_string())
    } else if chars[*pos] == '"' {
        *pos += 1;
        let mut s = String::new();
        while *pos < chars.len() {
            if chars[*pos] == '\\' { *pos += 1; if *pos < chars.len() { s.push(chars[*pos]); *pos += 1; } }
            else if chars[*pos] == '"' { *pos += 1; break; }
            else { s.push(chars[*pos]); *pos += 1; }
        }
        Ok(Sexpr::Str(s))
    } else if chars[*pos] == '-' || chars[*pos].is_ascii_digit() {
        let mut s = String::new();
        while *pos < chars.len() && (chars[*pos] == '-' || chars[*pos] == '.' || chars[*pos].is_ascii_digit()) {
            s.push(chars[*pos]); *pos += 1;
        }
        if s.contains('.') { Ok(Sexpr::Float(s.parse().map_err(|e| format!("float: {}", e))?)) }
        else { Ok(Sexpr::Int(s.parse().map_err(|e| format!("int: {}", e))?)) }
    } else {
        let mut s = String::new();
        while *pos < chars.len() && !chars[*pos].is_whitespace() && chars[*pos] != ')' && chars[*pos] != '(' {
            s.push(chars[*pos]); *pos += 1;
        }
        Ok(Sexpr::Str(s))
    }
}

pub fn find_all<'a>(expr: &'a Sexpr, tag: &str) -> Vec<&'a Sexpr> {
    match expr {
        Sexpr::List(items) => {
            let mut r = Vec::new();
            for item in items {
                if let Sexpr::List(inner) = item {
                    if inner.first().map(|s| matches!(s, Sexpr::Str(t) if t == tag)).unwrap_or(false) {
                        r.push(item);
                    }
                    r.extend(find_all(item, tag));
                }
            }
            r
        }
        _ => vec![],
    }
}

pub fn get_str<'a>(expr: &'a Sexpr, idx: usize) -> Option<&'a str> {
    match expr {
        Sexpr::List(items) => items.get(idx).and_then(|s| match s { Sexpr::Str(t) => Some(t.as_str()), _ => None }),
        _ => None,
    }
}

pub fn get_int(expr: &Sexpr, idx: usize) -> Option<i64> {
    match expr {
        Sexpr::List(items) => items.get(idx).and_then(|s| match s { Sexpr::Int(n) => Some(*n), _ => None }),
        _ => None,
    }
}

pub fn get_float(expr: &Sexpr, idx: usize) -> Option<f64> {
    match expr {
        Sexpr::List(items) => items.get(idx).and_then(|s| match s { Sexpr::Float(n) => Some(*n), Sexpr::Int(n) => Some(*n as f64), _ => None }),
        _ => None,
    }
}

pub fn find_at(expr: &Sexpr) -> Option<(f64, f64)> {
    match expr {
        Sexpr::List(items) => {
            items.iter().find_map(|i| match i {
                Sexpr::List(inner) if inner.first().map(|s| matches!(s, Sexpr::Str(t) if t == "at")).unwrap_or(false) => {
                    let x = inner.get(1).and_then(|s| match s { Sexpr::Int(n) => Some(*n as f64), Sexpr::Float(n) => Some(*n), _ => None })?;
                    let y = inner.get(2).and_then(|s| match s { Sexpr::Int(n) => Some(*n as f64), Sexpr::Float(n) => Some(*n), _ => None })?;
                    Some((x, y))
                }
                _ => None,
            })
        }
        _ => None,
    }
}

pub fn find_attr(expr: &Sexpr, tag: &str) -> Option<(f64, f64)> {
    match expr {
        Sexpr::List(items) => {
            items.iter().find_map(|i| match i {
                Sexpr::List(inner) if inner.first().map(|s| matches!(s, Sexpr::Str(t) if t == tag)).unwrap_or(false) => {
                    let x = inner.get(1).and_then(|s| match s { Sexpr::Int(n) => Some(*n as f64), Sexpr::Float(n) => Some(*n), _ => None })?;
                    let y = inner.get(2).and_then(|s| match s { Sexpr::Int(n) => Some(*n as f64), Sexpr::Float(n) => Some(*n), _ => None })?;
                    Some((x, y))
                }
                _ => None,
            })
        }
        _ => None,
    }
}

pub fn find_attr_float(expr: &Sexpr, tag: &str) -> Option<f64> {
    match expr {
        Sexpr::List(items) => {
            items.iter().find_map(|i| match i {
                Sexpr::List(inner) if inner.first().map(|s| matches!(s, Sexpr::Str(t) if t == tag)).unwrap_or(false) => {
                    inner.get(1).and_then(|s| match s { Sexpr::Int(n) => Some(*n as f64), Sexpr::Float(n) => Some(*n), _ => None })
                }
                _ => None,
            })
        }
        _ => None,
    }
}

pub fn find_first_child(expr: &Sexpr, tag: &str) -> Option<String> {
    match expr {
        Sexpr::List(items) => {
            if items.first().map(|s| matches!(s, Sexpr::Str(t) if t == "property")).unwrap_or(false) {
                let name = items.get(1).and_then(|s| match s { Sexpr::Str(n) => Some(n.as_str()), _ => None })?;
                if name == tag {
                    return items.get(2).and_then(|s| match s { Sexpr::Str(v) => Some(v.clone()), Sexpr::Int(n) => Some(n.to_string()), _ => None });
                }
            }
            items.iter().find_map(|i| match i {
                Sexpr::List(inner) if inner.first().map(|s| matches!(s, Sexpr::Str(t) if t == "property")).unwrap_or(false) => {
                    let name = inner.get(1).and_then(|s| match s { Sexpr::Str(n) => Some(n.as_str()), _ => None })?;
                    if name == tag {
                        inner.get(2).and_then(|s| match s { Sexpr::Str(v) => Some(v.clone()), Sexpr::Int(n) => Some(n.to_string()), _ => None })
                    } else { None }
                }
                _ => None,
            })
        }
        _ => None,
    }
}

pub fn import_kicad_sch(path: &str) -> Result<HashMap<SheetId, SchematicSheet>, String> {
    let text = std::fs::read_to_string(path).map_err(|e| e.to_string())?;
    let parsed = parse_sexpr(&text)?;
    let root = Sexpr::List(parsed);

    let mut symbols = Arena::new();
    let mut wires = Arena::new();
    let mut junctions = Arena::new();
    let mut net_labels = Arena::new();
    let mut buses = Arena::new();
    let mut text_annotations = Arena::new();
    let mut hier_sheets = Arena::new();

    for expr in find_all(&root, "symbol") {
        let _lib_name = get_str(expr, 1).unwrap_or("Unknown").to_string();
        let refdes = find_first_child(expr, "Reference").unwrap_or_else(|| "U?".to_string());
        let value = find_first_child(expr, "Value").unwrap_or_default();
        let pos = find_at(expr).unwrap_or((0.0, 0.0));

        symbols.insert(SymbolInstance {
            component_id: ComponentId(uuid::Uuid::new_v4()),
            position: Point { x: pos.0, y: pos.1 },
            rotation: Rotation(0.0),
            mirrored: false,
            refdes,
            value,
        });
    }

    for wire in find_all(&root, "wire") {
        let pts: Vec<Point> = wire_points(wire);
        if pts.len() >= 2 {
            wires.insert(Wire {
                points: pts,
                net_id: None,
                width: 0.254,
            });
        }
    }

    for label in find_all(&root, "label") {
        let text = get_str(label, 1).unwrap_or("").to_string();
        let pos = find_at(label).unwrap_or((0.0, 0.0));
        net_labels.insert(NetLabel {
            name: text,
            position: Point { x: pos.0, y: pos.1 },
        });
    }

    for junction in find_all(&root, "junction") {
        let pos = find_at(junction);
        if let Some(p) = pos {
            junctions.insert(Junction {
                position: Point { x: p.0, y: p.1 },
            });
        }
    }

    for text_expr in find_all(&root, "text") {
        let text = get_str(text_expr, 1).unwrap_or("").to_string();
        let pos = find_at(text_expr).unwrap_or((0.0, 0.0));
        let size = get_float(text_expr, 2).unwrap_or(1.27);
        text_annotations.insert(TextAnnotation {
            text,
            position: Point { x: pos.0, y: pos.1 },
            size,
        });
    }

    for bus_expr in find_all(&root, "bus") {
        let name = get_str(bus_expr, 1).unwrap_or("BUS").to_string();
        let pts = wire_points(bus_expr);
        let members: Vec<String> = find_all(bus_expr, "member").iter()
            .filter_map(|m| get_str(m, 1).map(|s| s.to_string()))
            .collect();
        let bus_wires = if pts.len() >= 2 { vec![pts] } else { vec![] };
        buses.insert(Bus { name, members, wires: bus_wires });
    }

    for sheet_expr in find_all(&root, "sheet") {
        let name = find_first_child(sheet_expr, "Name")
            .or_else(|| get_str(sheet_expr, 1).map(|s| s.to_string()))
            .unwrap_or_else(|| "Sheet".to_string());
        let pos = find_at(sheet_expr).unwrap_or((0.0, 0.0));
        let size_xy = find_attr(sheet_expr, "size").unwrap_or((200.0, 150.0));
        let mut pins = Vec::new();
        for pin_expr in find_all(sheet_expr, "pin") {
            let pin_name = get_str(pin_expr, 1).unwrap_or("").to_string();
            let pin_pos = find_at(pin_expr).unwrap_or((0.0, 0.0));
            pins.push(HierarchicalPin {
                name: pin_name,
                pin_number: (pins.len() + 1).to_string(),
                position: Point { x: pin_pos.0, y: pin_pos.1 },
            });
        }
        let child_id = SheetId(1 + hier_sheets.len());
        hier_sheets.insert(HierarchicalSheet {
            id: SheetId(hier_sheets.len()),
            name,
            child_sheet_id: child_id,
            pins,
            position: Point { x: pos.0, y: pos.1 },
            size: Size { width: size_xy.0, height: size_xy.1 },
        });
    }

    let sheet = SchematicSheet {
        id: SheetId(0),
        name: String::new(),
        symbols,
        wires,
        junctions,
        net_labels,
        buses,
        text_annotations,
        hier_sheets,
        properties: HashMap::new(),
    };

    let mut result = HashMap::new();
    result.insert(sheet.id, sheet);
    Ok(result)
}

fn wire_points(expr: &Sexpr) -> Vec<Point> {
    match expr {
        Sexpr::List(items) => {
            items.iter().filter_map(|i| match i {
                Sexpr::List(inner) if inner.first().map(|s| matches!(s, Sexpr::Str(t) if t == "pts")).unwrap_or(false) => {
                    Some(inner.iter().filter_map(|pt| match pt {
                        Sexpr::List(xy) if xy.first().map(|s| matches!(s, Sexpr::Str(t) if t == "xy")).unwrap_or(false) => {
                            let x = xy.get(1).and_then(|s| match s { Sexpr::Int(n) => Some(*n as f64), Sexpr::Float(n) => Some(*n), _ => None })?;
                            let y = xy.get(2).and_then(|s| match s { Sexpr::Int(n) => Some(*n as f64), Sexpr::Float(n) => Some(*n), _ => None })?;
                            Some(Point { x, y })
                        }
                        _ => None,
                    }).collect::<Vec<_>>())
                }
                _ => None,
            }).flatten().collect()
        }
        _ => vec![],
    }
}

pub fn export_kicad_sch(sheet: &SchematicSheet, path: &str) -> Result<(), String> {
    let mut out = String::new();
    out.push_str("(kicad_sch (version 20230121) (generator \"TraceCraft\")\n");
    out.push_str("  (paper \"A4\")\n");

    for (_, sym) in sheet.symbols.iter() {
        out.push_str(&format!("  (symbol (lib_id {}) (at {} {} 0)\n",
            escape_sexpr(&sym.component_id.to_string()), sym.position.x, sym.position.y));
        out.push_str(&format!("    (property \"Reference\" {}) (property \"Value\" {})\n",
            escape_sexpr(&sym.refdes), escape_sexpr(&sym.value)));
        out.push_str("  )\n");
    }

    for (_, wire) in sheet.wires.iter() {
        if wire.points.len() >= 2 {
            out.push_str("  (wire (pts");
            for pt in &wire.points {
                out.push_str(&format!(" (xy {} {})", pt.x, pt.y));
            }
            out.push_str(&format!(") (layer Eco1) (width {}))\n", wire.width));
        }
    }

    for (_, label) in sheet.net_labels.iter() {
        out.push_str(&format!("  (label \"{}\" (at {} {} 0))\n",
            escape_sexpr(&label.name), label.position.x, label.position.y));
    }

    for (_, junction) in sheet.junctions.iter() {
        out.push_str(&format!("  (junction (at {} {}))\n", junction.position.x, junction.position.y));
    }

    for (_, text) in sheet.text_annotations.iter() {
        out.push_str(&format!("  (text \"{}\" (at {} {} 0) (layer \"F.SilkS\")\n",
            escape_sexpr(&text.text), text.position.x, text.position.y));
        out.push_str(&format!("    (effects (font (size {} {})))\n", text.size, text.size));
        out.push_str("  )\n");
    }

    for (_, bus) in sheet.buses.iter() {
        for wire in &bus.wires {
            if wire.len() >= 2 {
                out.push_str("  (bus (pts");
                for pt in wire {
                    out.push_str(&format!(" (xy {} {})", pt.x, pt.y));
                }
                out.push_str(")");
            }
        }
        if !bus.members.is_empty() {
            out.push_str(" (members");
            for m in &bus.members {
                out.push_str(&format!(" (member {})", escape_sexpr(m)));
            }
            out.push_str(")");
        }
        out.push_str(")\n");
    }

    for (_, sheet_ref) in sheet.hier_sheets.iter() {
        out.push_str(&format!("  (sheet (at {} {} 0) (size {} {})\n",
            sheet_ref.position.x, sheet_ref.position.y,
            sheet_ref.size.width, sheet_ref.size.height));
        out.push_str(&format!("    (name \"\") (sheet_id \"\")\n"));
        for pin in &sheet_ref.pins {
            out.push_str(&format!("    (pin {} (at {} {} 0) (name {}) (number {})\n",
                "passive", pin.position.x, pin.position.y,
                escape_sexpr(&pin.name), pin.pin_number));
        }
        out.push_str("  )\n");
    }

    out.push_str(")\n");
    std::fs::write(path, out).map_err(|e| e.to_string())?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_simple_sexpr() {
        let input = "(kicad_sch (version 20230121))";
        let parsed = parse_sexpr(input).unwrap();
        assert_eq!(parsed.len(), 1);
        if let Sexpr::List(items) = &parsed[0] {
            assert_eq!(items.len(), 2);
            assert!(matches!(&items[0], Sexpr::Str(s) if s == "kicad_sch"));
        } else { panic!("Expected list"); }
    }

    #[test]
    fn test_parse_nested() {
        let input = "(symbol (lib Resistor) (at 100 200 0))";
        let parsed = parse_sexpr(input).unwrap();
        assert_eq!(parsed.len(), 1);
        if let Sexpr::List(items) = &parsed[0] {
            assert!(matches!(&items[0], Sexpr::Str(s) if s == "symbol"));
            assert!(find_at(&parsed[0]).is_some());
            let pos = find_at(&parsed[0]).unwrap();
            assert_eq!(pos.0, 100.0);
            assert_eq!(pos.1, 200.0);
        } else { panic!("Expected list"); }
    }

    #[test]
    fn test_parse_string_with_spaces() {
        let input = r#"(property "Reference" "R1")"#;
        let parsed = parse_sexpr(input).unwrap();
        assert_eq!(parsed.len(), 1);
        let val = find_first_child(&parsed[0], "Reference");
        assert_eq!(val, Some("R1".to_string()));
    }

    #[test]
    fn test_escape_roundtrip() {
        let label = "VCC_3V3";
        let escaped = escape_sexpr(label);
        assert_eq!(escaped, "VCC_3V3");
        let label2 = "Net (Label)";
        let escaped2 = escape_sexpr(label2);
        assert!(escaped2.contains('"'));
    }

    #[test]
    fn test_find_all() {
        let input = "(root (wire (pts (xy 0 0) (xy 10 10))) (wire (pts (xy 20 20) (xy 30 30))))";
        let parsed = parse_sexpr(input).unwrap();
        let root = Sexpr::List(parsed);
        let wires = find_all(&root, "wire");
        assert_eq!(wires.len(), 2);
    }

    #[test]
    fn test_get_str_and_float() {
        let input = "(test \"hello\" 3.14)";
        let parsed = parse_sexpr(input).unwrap();
        assert_eq!(parsed.len(), 1);
        assert_eq!(get_str(&parsed[0], 0), Some("test"));
        assert_eq!(get_str(&parsed[0], 1), Some("hello"));
        let f = get_float(&parsed[0], 2);
        assert!(f.is_some());
        assert!((f.unwrap() - 3.14).abs() < 1e-6);
    }
}
