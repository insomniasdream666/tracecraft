use serde::{Deserialize, Serialize};
use crate::model::schematic::*;
use crate::model::geometry::*;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Severity { Error, Warning }

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ERCRule {
    FloatingPin,
    UnconnectedNet,
    ShortedOutput,
    DuplicateRefdes,
    MissingRefdes,
    UnconnectedWire,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ERCViolation {
    pub severity: Severity,
    pub rule: ERCRule,
    pub description: String,
    pub location: Point,
    pub elements: Vec<(String, usize)>,
}

/// Run Electrical Rule Check on a schematic sheet.
/// Returns all violations found.
pub fn run_erc(sheet: &SchematicSheet) -> Vec<ERCViolation> {
    let mut violations = Vec::new();

    // 1. Check for duplicate reference designators
    let mut refdes_counts: std::collections::HashMap<&str, Vec<usize>> = std::collections::HashMap::new();
    for (id, sym) in sheet.symbols.iter() {
        if !sym.refdes.is_empty() {
            refdes_counts.entry(&sym.refdes).or_default().push(id);
        }
    }
    for (refdes, ids) in &refdes_counts {
        if ids.len() > 1 {
            violations.push(ERCViolation {
                severity: Severity::Error,
                rule: ERCRule::DuplicateRefdes,
                description: format!("Duplicate reference designator '{}' appears {} times", refdes, ids.len()),
                location: sheet.symbols.get(ids[0]).map(|s| s.position).unwrap_or(Point::new(0.0, 0.0)),
                elements: ids.iter().map(|&id| ("symbol".into(), id)).collect(),
            });
        }
    }

    // 2. Check for missing reference designators
    for (id, sym) in sheet.symbols.iter() {
        if sym.refdes.is_empty() {
            violations.push(ERCViolation {
                severity: Severity::Warning,
                rule: ERCRule::MissingRefdes,
                description: "Symbol has no reference designator".into(),
                location: sym.position,
                elements: vec![("symbol".into(), id)],
            });
        }
    }

    // 3. Check for floating pins (symbols with no connected wires)
    let mut connected_pins = std::collections::HashSet::new();
    for (_, wire) in sheet.wires.iter() {
        for pt in &wire.points {
            for (id, sym) in sheet.symbols.iter() {
                if pt.distance(&sym.position) < 10.0 {
                    connected_pins.insert(id);
                }
            }
        }
    }
    for (id, sym) in sheet.symbols.iter() {
        if !connected_pins.contains(&id) {
            violations.push(ERCViolation {
                severity: Severity::Warning,
                rule: ERCRule::FloatingPin,
                description: format!("Symbol '{}' has no connected wires", sym.refdes),
                location: sym.position,
                elements: vec![("symbol".into(), id)],
            });
        }
    }

    // 4. Check for unconnected wires (wires not connected to any symbol)
    for (id, wire) in sheet.wires.iter() {
        let mut connected = false;
        for pt in &wire.points {
            for (_, sym) in sheet.symbols.iter() {
                if pt.distance(&sym.position) < 10.0 {
                    connected = true;
                    break;
                }
            }
            if connected { break; }
        }
        if !connected {
            violations.push(ERCViolation {
                severity: Severity::Warning,
                rule: ERCRule::UnconnectedWire,
                description: "Wire is not connected to any symbol".into(),
                location: wire.points.first().copied().unwrap_or(Point::new(0.0, 0.0)),
                elements: vec![("wire".into(), id)],
            });
        }
    }

    // 5. Check for unconnected nets (net labels with no wires)
    for (id, label) in sheet.net_labels.iter() {
        let mut has_wire = false;
        for (_, wire) in sheet.wires.iter() {
            for pt in &wire.points {
                if pt.distance(&label.position) < 5.0 {
                    has_wire = true;
                    break;
                }
            }
            if has_wire { break; }
        }
        if !has_wire {
            violations.push(ERCViolation {
                severity: Severity::Warning,
                rule: ERCRule::UnconnectedNet,
                description: format!("Net label '{}' has no connected wires", label.name),
                location: label.position,
                elements: vec![("net_label".into(), id)],
            });
        }
    }

    violations
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::arena::Arena;
    use crate::model::id::{SheetId, ComponentId};
    use crate::model::geometry::Rotation;

    fn make_sheet(symbols: Arena<SymbolInstance>, wires: Arena<Wire>, labels: Arena<NetLabel>) -> SchematicSheet {
        SchematicSheet {
            id: SheetId(0),
            name: String::new(),
            symbols,
            wires,
            junctions: Arena::new(),
            net_labels: labels,
            buses: Arena::new(),
            text_annotations: Arena::new(),
            hier_sheets: Arena::new(),
            properties: std::collections::HashMap::new(),
        }
    }

    #[test]
    fn test_no_violations_on_empty_sheet() {
        let sheet = make_sheet(Arena::new(), Arena::new(), Arena::new());
        let violations = run_erc(&sheet);
        assert!(violations.is_empty());
    }

    #[test]
    fn test_duplicate_refdes() {
        let mut symbols = Arena::new();
        let id1 = symbols.insert(SymbolInstance {
            component_id: ComponentId(uuid::Uuid::new_v4()),
            position: Point::new(0.0, 0.0),
            rotation: Rotation(0.0), mirrored: false,
            refdes: "R1".into(), value: "".into(),
        });
        let id2 = symbols.insert(SymbolInstance {
            component_id: ComponentId(uuid::Uuid::new_v4()),
            position: Point::new(100.0, 100.0),
            rotation: Rotation(0.0), mirrored: false,
            refdes: "R1".into(), value: "".into(),
        });
        let sheet = make_sheet(symbols, Arena::new(), Arena::new());
        let violations = run_erc(&sheet);
        assert!(violations.iter().any(|v| matches!(v.rule, ERCRule::DuplicateRefdes)));
    }

    #[test]
    fn test_missing_refdes() {
        let mut symbols = Arena::new();
        symbols.insert(SymbolInstance {
            component_id: ComponentId(uuid::Uuid::new_v4()),
            position: Point::new(0.0, 0.0),
            rotation: Rotation(0.0), mirrored: false,
            refdes: "".into(), value: "".into(),
        });
        let sheet = make_sheet(symbols, Arena::new(), Arena::new());
        let violations = run_erc(&sheet);
        assert!(violations.iter().any(|v| matches!(v.rule, ERCRule::MissingRefdes)));
    }

    #[test]
    fn test_floating_pin() {
        let mut symbols = Arena::new();
        symbols.insert(SymbolInstance {
            component_id: ComponentId(uuid::Uuid::new_v4()),
            position: Point::new(50.0, 50.0),
            rotation: Rotation(0.0), mirrored: false,
            refdes: "R1".into(), value: "".into(),
        });
        let sheet = make_sheet(symbols, Arena::new(), Arena::new());
        let violations = run_erc(&sheet);
        assert!(violations.iter().any(|v| matches!(v.rule, ERCRule::FloatingPin)));
    }

    #[test]
    fn test_connected_symbol_no_floating() {
        let mut symbols = Arena::new();
        symbols.insert(SymbolInstance {
            component_id: ComponentId(uuid::Uuid::new_v4()),
            position: Point::new(50.0, 50.0),
            rotation: Rotation(0.0), mirrored: false,
            refdes: "R1".into(), value: "".into(),
        });
        let mut wires = Arena::new();
        wires.insert(Wire {
            points: vec![Point::new(50.0, 50.0), Point::new(150.0, 50.0)],
            net_id: None, width: 0.254,
        });
        let sheet = make_sheet(symbols, wires, Arena::new());
        let violations = run_erc(&sheet);
        assert!(!violations.iter().any(|v| matches!(v.rule, ERCRule::FloatingPin)));
    }

    #[test]
    fn test_unconnected_net_label() {
        let mut labels = Arena::new();
        labels.insert(NetLabel {
            name: "VCC".into(),
            position: Point::new(100.0, 100.0),
        });
        let sheet = make_sheet(Arena::new(), Arena::new(), labels);
        let violations = run_erc(&sheet);
        assert!(violations.iter().any(|v| matches!(v.rule, ERCRule::UnconnectedNet)));
    }
}
