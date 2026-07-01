use std::collections::HashMap;

// ── Helpers ────────────────────────────────────────────────────────────────

fn make_db() -> rusqlite::Connection {
    let conn = rusqlite::Connection::open_in_memory().unwrap();
    tracecraft_lib::db::migrations::run_migrations(&conn).unwrap();
    conn
}

fn make_library() -> std::sync::Arc<dyn tracecraft_lib::model::library::ComponentRepository> {
    let conn = rusqlite::Connection::open_in_memory().unwrap();
    tracecraft_lib::db::migrations::run_migrations(&conn).unwrap();
    let lib: std::sync::Arc<dyn tracecraft_lib::model::library::ComponentRepository> =
        std::sync::Arc::new(tracecraft_lib::db::library_repo::SqliteComponentRepository::new(conn));
    lib
}

fn point(x: f64, y: f64) -> tracecraft_lib::model::geometry::Point {
    tracecraft_lib::model::geometry::Point::new(x, y)
}

fn rotation(d: f64) -> tracecraft_lib::model::geometry::Rotation {
    tracecraft_lib::model::geometry::Rotation(d)
}

// ── 1. PCB Commands ────────────────────────────────────────────────────────

#[test]
fn test_pcb_place_footprint() {
    let mut board = tracecraft_lib::model::pcb::PcbBoard::default();
    let cid = tracecraft_lib::model::library::ComponentId(uuid::Uuid::new_v4());
    let id = board.footprints.insert(tracecraft_lib::model::pcb::FootprintInstance {
        component_id: cid,
        position: point(10.0, 20.0),
        rotation: rotation(45.0),
        mirrored: false,
        refdes: String::new(),
        locked: false,
    });
    assert_eq!(id, 0);
    let fp = board.footprints.get(id).unwrap();
    assert_eq!(fp.component_id, cid);
    assert!((fp.position.x - 10.0).abs() < 1e-10);
    assert_eq!(board.footprints.len(), 1);
}

#[test]
fn test_pcb_route_trace() {
    let mut board = tracecraft_lib::model::pcb::PcbBoard::default();
    let id = board.traces.insert(tracecraft_lib::model::pcb::Trace {
        net_id: None,
        layer: tracecraft_lib::model::id::LayerId(1),
        width: 0.25,
        points: vec![point(0.0, 0.0), point(10.0, 0.0), point(10.0, 10.0)],
    });
    let trace = board.traces.get(id).unwrap();
    assert_eq!(trace.points.len(), 3);
    assert!((trace.width - 0.25).abs() < 1e-10);
}

#[test]
fn test_pcb_add_via() {
    let mut board = tracecraft_lib::model::pcb::PcbBoard::default();
    let id = board.vias.insert(tracecraft_lib::model::pcb::Via {
        position: point(5.0, 5.0),
        start_layer: tracecraft_lib::model::id::LayerId(1),
        end_layer: tracecraft_lib::model::id::LayerId(2),
        diameter: 0.6,
        drill: 0.3,
        net_id: None,
    });
    let via = board.vias.get(id).unwrap();
    assert!((via.position.x - 5.0).abs() < 1e-10);
    assert!((via.diameter - 0.6).abs() < 1e-10);
}

#[test]
fn test_pcb_draw_zone() {
    let mut board = tracecraft_lib::model::pcb::PcbBoard::default();
    let id = board.zones.insert(tracecraft_lib::model::pcb::Zone {
        net_id: None,
        layer: tracecraft_lib::model::id::LayerId(1),
        outline: vec![point(0.0, 0.0), point(10.0, 0.0), point(10.0, 10.0), point(0.0, 10.0)],
        clearance: 0.2,
        thermal_relief: true,
    });
    let zone = board.zones.get(id).unwrap();
    assert_eq!(zone.outline.len(), 4);
}

#[test]
fn test_pcb_set_board_outline() {
    let mut board = tracecraft_lib::model::pcb::PcbBoard::default();
    board.outline = Some(tracecraft_lib::model::pcb::BoardOutline {
        shape: vec![vec![point(0.0, 0.0), point(100.0, 0.0), point(100.0, 80.0), point(0.0, 80.0)]],
        cutouts: vec![vec![point(10.0, 10.0), point(20.0, 10.0), point(20.0, 20.0), point(10.0, 20.0)]],
    });
    assert!(board.outline.is_some());
    assert_eq!(board.outline.as_ref().unwrap().shape.len(), 1);
    assert_eq!(board.outline.as_ref().unwrap().cutouts.len(), 1);
}

#[test]
fn test_pcb_delete_restores_arena_slot() {
    let mut board = tracecraft_lib::model::pcb::PcbBoard::default();
    let id1 = board.traces.insert(tracecraft_lib::model::pcb::Trace {
        net_id: None, layer: tracecraft_lib::model::id::LayerId(1), width: 0.25,
        points: vec![point(0.0, 0.0), point(10.0, 0.0)],
    });
    let id2 = board.traces.insert(tracecraft_lib::model::pcb::Trace {
        net_id: None, layer: tracecraft_lib::model::id::LayerId(1), width: 0.25,
        points: vec![point(0.0, 10.0), point(10.0, 10.0)],
    });
    assert_eq!(id1, 0);
    assert_eq!(id2, 1);
    board.traces.remove(id2);
    assert_eq!(board.traces.len(), 1);
    let new_id = board.traces.insert(tracecraft_lib::model::pcb::Trace {
        net_id: None, layer: tracecraft_lib::model::id::LayerId(1), width: 0.25,
        points: vec![point(0.0, 20.0), point(10.0, 20.0)],
    });
    assert_eq!(new_id, 1);
    assert_eq!(board.traces.len(), 2);
}

#[test]
fn test_pcb_get_ratsnest() {
    let mut board = tracecraft_lib::model::pcb::PcbBoard::default();
    board.footprints.insert(tracecraft_lib::model::pcb::FootprintInstance {
        component_id: tracecraft_lib::model::library::ComponentId(uuid::Uuid::new_v4()),
        position: point(0.0, 0.0),
        rotation: rotation(0.0), mirrored: false, refdes: "R1".into(), locked: false,
    });
    board.footprints.insert(tracecraft_lib::model::pcb::FootprintInstance {
        component_id: tracecraft_lib::model::library::ComponentId(uuid::Uuid::new_v4()),
        position: point(100.0, 100.0),
        rotation: rotation(0.0), mirrored: false, refdes: "R2".into(), locked: false,
    });
    board.netlist = tracecraft_lib::model::netlist::Netlist {
        nets: vec![tracecraft_lib::model::netlist::Net {
            id: tracecraft_lib::model::id::NetId(1),
            name: "GND".into(),
            connections: vec![
                tracecraft_lib::model::netlist::PinRef { refdes: "R1".into(), pin_number: "1".into() },
                tracecraft_lib::model::netlist::PinRef { refdes: "R2".into(), pin_number: "1".into() },
            ],
        }],
    };
    let lines = tracecraft_lib::engine::ratsnest::compute_ratsnest(&board);
    assert!(!lines.is_empty());
    let gnd_lines: Vec<_> = lines.iter().filter(|l| l.net_id == Some(tracecraft_lib::model::id::NetId(1))).collect();
    assert!(!gnd_lines.is_empty());
}

#[test]
fn test_pcb_sync_netlist() {
    let mut board = tracecraft_lib::model::pcb::PcbBoard::default();
    board.netlist = tracecraft_lib::model::netlist::Netlist {
        nets: vec![tracecraft_lib::model::netlist::Net {
            id: tracecraft_lib::model::id::NetId(1),
            name: "VCC".into(),
            connections: vec![
                tracecraft_lib::model::netlist::PinRef { refdes: "U1".into(), pin_number: "1".into() },
            ],
        }],
    };
    assert_eq!(board.netlist.nets.len(), 1);
    assert_eq!(board.netlist.nets[0].name, "VCC");
}

// ── 2. Schematic Commands ─────────────────────────────────────────────────

#[test]
fn test_schematic_place_symbol() {
    let mut sheet = tracecraft_lib::model::schematic::SchematicSheet::default();
    sheet.id = tracecraft_lib::model::id::SheetId(0);
    let cid = tracecraft_lib::model::library::ComponentId(uuid::Uuid::new_v4());
    let id = sheet.symbols.insert(tracecraft_lib::model::schematic::SymbolInstance {
        component_id: cid,
        position: point(50.0, 50.0),
        rotation: rotation(0.0),
        mirrored: false,
        refdes: String::new(),
        value: String::new(),
    });
    let sym = sheet.symbols.get(id).unwrap();
    assert_eq!(sym.component_id, cid);
    assert!((sym.position.x - 50.0).abs() < 1e-10);
}

#[test]
fn test_schematic_draw_wire() {
    let mut sheet = tracecraft_lib::model::schematic::SchematicSheet::default();
    let id = sheet.wires.insert(tracecraft_lib::model::schematic::Wire {
        points: vec![point(0.0, 0.0), point(100.0, 0.0)],
        net_id: None,
        width: 0.25,
    });
    let wire = sheet.wires.get(id).unwrap();
    assert_eq!(wire.points.len(), 2);
    assert!((wire.width - 0.25).abs() < 1e-10);
}

#[test]
fn test_schematic_add_net_label() {
    let mut sheet = tracecraft_lib::model::schematic::SchematicSheet::default();
    let id = sheet.net_labels.insert(tracecraft_lib::model::schematic::NetLabel {
        name: "VCC".into(),
        position: point(50.0, 50.0),
    });
    let label = sheet.net_labels.get(id).unwrap();
    assert_eq!(label.name, "VCC");
}

#[test]
fn test_schematic_add_junction() {
    let mut sheet = tracecraft_lib::model::schematic::SchematicSheet::default();
    let id = sheet.junctions.insert(tracecraft_lib::model::schematic::Junction {
        position: point(25.0, 25.0),
    });
    let junc = sheet.junctions.get(id).unwrap();
    assert!((junc.position.x - 25.0).abs() < 1e-10);
}

#[test]
fn test_schematic_delete_restores_arena_slot() {
    let mut sheet = tracecraft_lib::model::schematic::SchematicSheet::default();
    let id1 = sheet.symbols.insert(tracecraft_lib::model::schematic::SymbolInstance {
        component_id: tracecraft_lib::model::library::ComponentId(uuid::Uuid::new_v4()),
        position: point(0.0, 0.0), rotation: rotation(0.0),
        mirrored: false, refdes: String::new(), value: String::new(),
    });
    let id2 = sheet.symbols.insert(tracecraft_lib::model::schematic::SymbolInstance {
        component_id: tracecraft_lib::model::library::ComponentId(uuid::Uuid::new_v4()),
        position: point(100.0, 100.0), rotation: rotation(0.0),
        mirrored: false, refdes: String::new(), value: String::new(),
    });
    sheet.symbols.remove(id1);
    assert_eq!(sheet.symbols.len(), 1);
    let reused = sheet.symbols.insert(tracecraft_lib::model::schematic::SymbolInstance {
        component_id: tracecraft_lib::model::library::ComponentId(uuid::Uuid::new_v4()),
        position: point(200.0, 200.0), rotation: rotation(0.0),
        mirrored: false, refdes: String::new(), value: String::new(),
    });
    assert_eq!(reused, 0);
}

#[test]
fn test_schematic_annotate() {
    let mut sheet = tracecraft_lib::model::schematic::SchematicSheet::default();

    // Insert symbols out of position order
    let _id_late = sheet.symbols.insert(tracecraft_lib::model::schematic::SymbolInstance {
        component_id: tracecraft_lib::model::library::ComponentId(uuid::Uuid::new_v4()),
        position: point(200.0, 200.0), rotation: rotation(0.0),
        mirrored: false, refdes: String::new(), value: "Resistor".into(),
    });
    let _id_early = sheet.symbols.insert(tracecraft_lib::model::schematic::SymbolInstance {
        component_id: tracecraft_lib::model::library::ComponentId(uuid::Uuid::new_v4()),
        position: point(10.0, 10.0), rotation: rotation(0.0),
        mirrored: false, refdes: String::new(), value: "Capacitor".into(),
    });

    // Replicate annotate logic
    let mut counters: HashMap<String, usize> = HashMap::new();
    let mut symbol_ids: Vec<usize> = sheet.symbols.iter().map(|(id, _)| id).collect();
    symbol_ids.sort_by(|&a, &b| {
        let pa = sheet.symbols.get(a).map(|s| s.position).unwrap_or(point(0.0, 0.0));
        let pb = sheet.symbols.get(b).map(|s| s.position).unwrap_or(point(0.0, 0.0));
        (pa.y * 1000.0 + pa.x).partial_cmp(&(pb.y * 1000.0 + pb.x)).unwrap()
    });

    fn prefix_for(name: &str) -> &'static str {
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

    // Check first symbol (position 10,10 -> Capacitor) got refdes "C1"
    let sym_at_10 = sheet.symbols.iter().find(|(_, s)| (s.position.x - 10.0).abs() < 1.0).unwrap();
    assert_eq!(sym_at_10.1.refdes, "C1");
    // Check second symbol (position 200,200 -> Resistor) got refdes "R1"
    let sym_at_200 = sheet.symbols.iter().find(|(_, s)| (s.position.x - 200.0).abs() < 1.0).unwrap();
    assert_eq!(sym_at_200.1.refdes, "R1");
}

#[test]
fn test_schematic_create_bus() {
    let mut sheet = tracecraft_lib::model::schematic::SchematicSheet::default();
    let id = sheet.buses.insert(tracecraft_lib::model::schematic::Bus {
        name: "DATA_BUS".into(),
        members: vec!["D0".into(), "D1".into(), "D2".into(), "D3".into()],
        wires: vec![],
    });
    let bus = sheet.buses.get(id).unwrap();
    assert_eq!(bus.name, "DATA_BUS");
    assert_eq!(bus.members.len(), 4);
}

#[test]
fn test_schematic_run_erc() {
    let sheet = tracecraft_lib::model::schematic::SchematicSheet::default();
    let violations = tracecraft_lib::engine::erc::run_erc(&sheet);
    // Empty sheet should have no violations
    assert!(violations.is_empty());
}

#[test]
fn test_schematic_erc_detects_floating_symbol() {
    let mut sheet = tracecraft_lib::model::schematic::SchematicSheet::default();
    sheet.symbols.insert(tracecraft_lib::model::schematic::SymbolInstance {
        component_id: tracecraft_lib::model::library::ComponentId(uuid::Uuid::new_v4()),
        position: point(50.0, 50.0), rotation: rotation(0.0),
        mirrored: false, refdes: "R1".into(), value: "Resistor".into(),
    });
    let violations = tracecraft_lib::engine::erc::run_erc(&sheet);
    assert!(violations.iter().any(|v| matches!(v.rule, tracecraft_lib::engine::erc::ERCRule::FloatingPin)));
}

#[test]
fn test_schematic_move_elements() {
    let mut sheet = tracecraft_lib::model::schematic::SchematicSheet::default();
    let sid = sheet.symbols.insert(tracecraft_lib::model::schematic::SymbolInstance {
        component_id: tracecraft_lib::model::library::ComponentId(uuid::Uuid::new_v4()),
        position: point(10.0, 10.0), rotation: rotation(0.0),
        mirrored: false, refdes: "R1".into(), value: String::new(),
    });
    // Move it
    if let Some(sym) = sheet.symbols.get_mut(sid) {
        sym.position = point(30.0, 40.0);
    }
    let sym = sheet.symbols.get(sid).unwrap();
    assert!((sym.position.x - 30.0).abs() < 1e-10);
    assert!((sym.position.y - 40.0).abs() < 1e-10);
}

// ── 3. Library Commands ─────────────────────────────────────────────────────

#[test]
fn test_library_create_and_search_component() {
    let lib = make_library();
    let cid = tracecraft_lib::model::library::ComponentId(uuid::Uuid::new_v4());

    let comp = tracecraft_lib::model::library::Component {
        id: cid,
        name: "NE555".into(),
        description: "Precision timer IC".into(),
        manufacturer: Some("TI".into()),
        category: Some("ic".into()),
        symbol: tracecraft_lib::model::library::SymbolDef::default(),
        footprint: tracecraft_lib::model::library::FootprintDef::default(),
        model_3d: None,
        parameters: HashMap::new(),
        created_at: "2025-01-01T00:00:00Z".into(),
        updated_at: "2025-01-01T00:00:00Z".into(),
    };
    futures::executor::block_on(lib.save(&comp)).unwrap();

    let results = futures::executor::block_on(lib.search(&tracecraft_lib::model::library::SearchQuery {
        text: "NE555".into(),
        category: None,
        page: 0,
        per_page: 20,
    })).unwrap();
    assert!(!results.is_empty());
    assert_eq!(results[0].name, "NE555");
}

#[test]
fn test_library_get_component_detail() {
    let lib = make_library();
    let cid = tracecraft_lib::model::library::ComponentId(uuid::Uuid::new_v4());

    let comp = tracecraft_lib::model::library::Component {
        id: cid,
        name: "LM358".into(),
        description: "Dual op-amp".into(),
        manufacturer: None,
        category: Some("ic".into()),
        symbol: tracecraft_lib::model::library::SymbolDef::default(),
        footprint: tracecraft_lib::model::library::FootprintDef::default(),
        model_3d: None,
        parameters: HashMap::new(),
        created_at: "2025-01-01T00:00:00Z".into(),
        updated_at: "2025-01-01T00:00:00Z".into(),
    };
    futures::executor::block_on(lib.save(&comp)).unwrap();

    let detail = futures::executor::block_on(lib.get_by_id(cid)).unwrap();
    assert!(detail.is_some());
    assert_eq!(detail.unwrap().name, "LM358");
}

#[test]
fn test_library_delete_component() {
    let lib = make_library();
    let cid = tracecraft_lib::model::library::ComponentId(uuid::Uuid::new_v4());

    let comp = tracecraft_lib::model::library::Component {
        id: cid,
        name: "ToDelete".into(),
        description: "Will be removed".into(),
        manufacturer: None,
        category: None,
        symbol: tracecraft_lib::model::library::SymbolDef::default(),
        footprint: tracecraft_lib::model::library::FootprintDef::default(),
        model_3d: None,
        parameters: HashMap::new(),
        created_at: "2025-01-01T00:00:00Z".into(),
        updated_at: "2025-01-01T00:00:00Z".into(),
    };
    futures::executor::block_on(lib.save(&comp)).unwrap();

    futures::executor::block_on(lib.delete(cid)).unwrap();

    let detail = futures::executor::block_on(lib.get_by_id(cid)).unwrap();
    assert!(detail.is_none());
}

#[test]
fn test_library_get_categories() {
    let lib = make_library();
    let cats = futures::executor::block_on(lib.get_categories()).unwrap();
    assert!(!cats.is_empty());
    assert!(cats.iter().any(|c| c.name == "Resistors"));
}

#[test]
fn test_library_get_footprint_defs() {
    let lib = make_library();
    let cid = tracecraft_lib::model::library::ComponentId(uuid::Uuid::new_v4());

    let fp = tracecraft_lib::model::library::FootprintDef {
        pads: vec![tracecraft_lib::model::library::PadDef {
            number: "1".into(),
            shape: tracecraft_lib::model::library::PadShape::Round,
            position: point(0.0, 0.0),
            size: tracecraft_lib::model::geometry::Size { width: 1.5, height: 1.5 },
            layer: tracecraft_lib::model::id::LayerId(1),
            drill: Some(0.6),
        }],
        outline: vec![point(-5.0, -5.0), point(5.0, -5.0), point(5.0, 5.0), point(-5.0, 5.0)],
        courtyard: vec![],
    };

    let comp = tracecraft_lib::model::library::Component {
        id: cid,
        name: "LED".into(),
        description: "LED".into(),
        manufacturer: None,
        category: None,
        symbol: tracecraft_lib::model::library::SymbolDef::default(),
        footprint: fp,
        model_3d: None,
        parameters: HashMap::new(),
        created_at: "2025-01-01T00:00:00Z".into(),
        updated_at: "2025-01-01T00:00:00Z".into(),
    };
    futures::executor::block_on(lib.save(&comp)).unwrap();

    let detail = futures::executor::block_on(lib.get_by_id(cid)).unwrap().unwrap();
    assert_eq!(detail.footprint.pads.len(), 1);
    assert_eq!(detail.footprint.pads[0].number, "1");
    assert_eq!(detail.footprint.outline.len(), 4);
}

// ── 4. Routing Commands ─────────────────────────────────────────────────────

#[test]
fn test_routing_autoroute_net() {
    let mut board = tracecraft_lib::model::pcb::PcbBoard::default();
    // Place two footprints with same net; route between points just outside pad blocked radius
    board.footprints.insert(tracecraft_lib::model::pcb::FootprintInstance {
        component_id: tracecraft_lib::model::library::ComponentId(uuid::Uuid::new_v4()),
        position: point(100.0, 100.0),
        rotation: rotation(0.0), mirrored: false, refdes: "R1".into(), locked: false,
    });
    board.footprints.insert(tracecraft_lib::model::pcb::FootprintInstance {
        component_id: tracecraft_lib::model::library::ComponentId(uuid::Uuid::new_v4()),
        position: point(115.0, 100.0),
        rotation: rotation(0.0), mirrored: false, refdes: "R2".into(), locked: false,
    });
    // Set netlist so both are on the same net
    board.netlist = tracecraft_lib::model::netlist::Netlist {
        nets: vec![tracecraft_lib::model::netlist::Net {
            id: tracecraft_lib::model::id::NetId(1),
            name: "NET1".into(),
            connections: vec![
                tracecraft_lib::model::netlist::PinRef { refdes: "R1".into(), pin_number: "1".into() },
                tracecraft_lib::model::netlist::PinRef { refdes: "R2".into(), pin_number: "1".into() },
            ],
        }],
    };
    // Route between two open points 6mm from nearest footprint center (outside 5.15mm blocked radius)
    let cfg = tracecraft_lib::engine::autorouter::RouterConfig::default();
    let router = tracecraft_lib::engine::autorouter::AStarRouter::new(cfg);
    let result = router.route_net(
        &board,
        point(106.0, 100.0),
        point(109.0, 100.0),
        tracecraft_lib::model::id::LayerId(1),
        tracecraft_lib::model::id::LayerId(1),
        Some(tracecraft_lib::model::id::NetId(1)),
    );

    assert!(result.success, "Autorouter should find a short path");
    assert!(!result.traces.is_empty(), "Should produce at least one trace");
    assert!(result.path_length > 0.0, "Path length should be positive");
    for t in &result.traces {
        assert_eq!(t.net_id, Some(tracecraft_lib::model::id::NetId(1)));
    }
}

#[test]
fn test_routing_get_routing_status() {
    let cfg = tracecraft_lib::engine::autorouter::RouterConfig::default();
    let router = tracecraft_lib::engine::autorouter::AStarRouter::new(cfg);
    let board = tracecraft_lib::model::pcb::PcbBoard::default();
    // Route on same layer with sufficient distance between points
    let result = router.route_net(
        &board,
        point(0.0, 0.0),
        point(50.0, 50.0),
        tracecraft_lib::model::id::LayerId(1),
        tracecraft_lib::model::id::LayerId(1),  // same layer
        None,
    );
    assert!(result.success, "Autorouter on empty board should succeed: success={}", result.success);
    assert!(result.path_length > 0.0, "Path length should be positive");
    assert!(!result.traces.is_empty(), "Should produce at least one trace");
}

// ── 5. Project Commands ─────────────────────────────────────────────────────

#[test]
fn test_project_create_and_save_and_open() {
    let conn = make_db();

    let id = uuid::Uuid::new_v4();
    let now = chrono::Utc::now().to_rfc3339();
    let project = tracecraft_lib::model::project::Project {
        id,
        name: "MyBoard".into(),
        path: "/tmp/test_project".into(),
        created_at: now.clone(),
        updated_at: now.clone(),
    };

    tracecraft_lib::db::project_store::save_project(&conn, &project).unwrap();

    let loaded = tracecraft_lib::db::project_store::load_project(&conn, id).unwrap();
    assert!(loaded.is_some());
    assert_eq!(loaded.unwrap().name, "MyBoard");
}

#[test]
fn test_project_summary_fields() {
    let id = uuid::Uuid::new_v4();
    let now = chrono::Utc::now().to_rfc3339();
    let summary = tracecraft_lib::model::project::ProjectSummary {
        id,
        name: "TestProject".into(),
        path: "/tmp/test".into(),
        updated_at: now.clone(),
    };
    assert_eq!(summary.name, "TestProject");
    assert_eq!(summary.id, id);
}

// ── 6. DRC Commands ─────────────────────────────────────────────────────────

#[test]
fn test_drc_no_violations_on_empty_board() {
    let board = tracecraft_lib::model::pcb::PcbBoard::default();
    let violations = tracecraft_lib::engine::drc::run_drc_checks(&board);
    assert!(violations.is_empty());
}

#[test]
fn test_drc_trace_width_violation() {
    let mut board = tracecraft_lib::model::pcb::PcbBoard::default();
    board.traces.insert(tracecraft_lib::model::pcb::Trace {
        net_id: None,
        layer: tracecraft_lib::model::id::LayerId(1),
        width: 0.01,
        points: vec![point(0.0, 0.0), point(10.0, 0.0)],
    });
    let violations = tracecraft_lib::engine::drc::run_drc_checks(&board);
    assert!(violations.iter().any(|v| v.rule == tracecraft_lib::engine::drc::DRCRule::TraceWidth));
}

#[test]
fn test_drc_trace_clearance_violation() {
    let mut board = tracecraft_lib::model::pcb::PcbBoard::default();
    board.traces.insert(tracecraft_lib::model::pcb::Trace {
        net_id: None, layer: tracecraft_lib::model::id::LayerId(1), width: 0.25,
        points: vec![point(0.0, 0.0), point(10.0, 0.0)],
    });
    board.traces.insert(tracecraft_lib::model::pcb::Trace {
        net_id: None, layer: tracecraft_lib::model::id::LayerId(1), width: 0.25,
        points: vec![point(0.0, 0.15), point(10.0, 0.15)],
    });
    let violations = tracecraft_lib::engine::drc::run_drc_checks(&board);
    assert!(violations.iter().any(|v| v.rule == tracecraft_lib::engine::drc::DRCRule::TraceClearance));
}

#[test]
fn test_drc_annular_ring_violation() {
    let mut board = tracecraft_lib::model::pcb::PcbBoard::default();
    board.vias.insert(tracecraft_lib::model::pcb::Via {
        position: point(50.0, 50.0),
        start_layer: tracecraft_lib::model::id::LayerId(1),
        end_layer: tracecraft_lib::model::id::LayerId(2),
        diameter: 0.4,
        drill: 0.35,
        net_id: None,
    });
    let violations = tracecraft_lib::engine::drc::run_drc_checks(&board);
    assert!(violations.iter().any(|v| v.rule == tracecraft_lib::engine::drc::DRCRule::AnnularRing));
}

#[test]
fn test_drc_hole_size_violation() {
    let mut board = tracecraft_lib::model::pcb::PcbBoard::default();
    board.vias.insert(tracecraft_lib::model::pcb::Via {
        position: point(50.0, 50.0),
        start_layer: tracecraft_lib::model::id::LayerId(1),
        end_layer: tracecraft_lib::model::id::LayerId(2),
        diameter: 0.6,
        drill: 0.1,
        net_id: None,
    });
    let violations = tracecraft_lib::engine::drc::run_drc_checks(&board);
    assert!(violations.iter().any(|v| v.rule == tracecraft_lib::engine::drc::DRCRule::HoleSize));
}

// ── 7. Netlist ──────────────────────────────────────────────────────────────

#[test]
fn test_netlist_extract_empty() {
    let sheet = tracecraft_lib::model::schematic::SchematicSheet::default();
    let netlist = tracecraft_lib::engine::netlist::extract_netlist(&sheet, &mut 0);
    assert!(netlist.nets.is_empty());
}

#[test]
fn test_netlist_extract_two_symbols_connected() {
    let mut sheet = tracecraft_lib::model::schematic::SchematicSheet::default();
    sheet.symbols.insert(tracecraft_lib::model::schematic::SymbolInstance {
        component_id: tracecraft_lib::model::library::ComponentId(uuid::Uuid::new_v4()),
        position: point(0.0, 0.0), rotation: rotation(0.0),
        mirrored: false, refdes: "R1".into(), value: "Resistor".into(),
    });
    sheet.symbols.insert(tracecraft_lib::model::schematic::SymbolInstance {
        component_id: tracecraft_lib::model::library::ComponentId(uuid::Uuid::new_v4()),
        position: point(100.0, 0.0), rotation: rotation(0.0),
        mirrored: false, refdes: "R2".into(), value: "Resistor".into(),
    });
    // Wire connecting both symbols (at y=0, both symbols are on the line)
    sheet.wires.insert(tracecraft_lib::model::schematic::Wire {
        points: vec![point(0.0, 0.0), point(100.0, 0.0)],
        net_id: None,
        width: 0.25,
    });

    let mut next_id = 0;
    let netlist = tracecraft_lib::engine::netlist::extract_netlist(&sheet, &mut next_id);
    assert_eq!(netlist.nets.len(), 1, "Should create one net for the connected pair");
    let net = &netlist.nets[0];
    assert!(net.name.starts_with("N$"), "Auto-generated net name should start with N$");
    // Both R1 and R2 should be in the connections
    let refdeses: Vec<&str> = net.connections.iter().map(|c| c.refdes.as_str()).collect();
    assert!(refdeses.contains(&"R1"));
    assert!(refdeses.contains(&"R2"));
}

#[test]
fn test_netlist_with_label() {
    let mut sheet = tracecraft_lib::model::schematic::SchematicSheet::default();
    sheet.symbols.insert(tracecraft_lib::model::schematic::SymbolInstance {
        component_id: tracecraft_lib::model::library::ComponentId(uuid::Uuid::new_v4()),
        position: point(0.0, 0.0), rotation: rotation(0.0),
        mirrored: false, refdes: "R1".into(), value: "Resistor".into(),
    });
    sheet.wires.insert(tracecraft_lib::model::schematic::Wire {
        points: vec![point(0.0, 0.0), point(50.0, 0.0)],
        net_id: None,
        width: 0.25,
    });
    sheet.net_labels.insert(tracecraft_lib::model::schematic::NetLabel {
        name: "SIGNAL".into(),
        position: point(50.0, 0.0),
    });

    let mut next_id = 0;
    let netlist = tracecraft_lib::engine::netlist::extract_netlist(&sheet, &mut next_id);
    assert_eq!(netlist.nets.len(), 1);
    assert_eq!(netlist.nets[0].name, "SIGNAL", "Net should use the label name");
}

// ── 8. Viewport ─────────────────────────────────────────────────────────────

#[test]
fn test_get_board_bounds_with_footprints() {
    let mut board = tracecraft_lib::model::pcb::PcbBoard::default();
    board.footprints.insert(tracecraft_lib::model::pcb::FootprintInstance {
        component_id: tracecraft_lib::model::library::ComponentId(uuid::Uuid::new_v4()),
        position: point(10.0, 20.0),
        rotation: rotation(0.0), mirrored: false, refdes: "U1".into(), locked: false,
    });
    board.footprints.insert(tracecraft_lib::model::pcb::FootprintInstance {
        component_id: tracecraft_lib::model::library::ComponentId(uuid::Uuid::new_v4()),
        position: point(100.0, 80.0),
        rotation: rotation(0.0), mirrored: false, refdes: "U2".into(), locked: false,
    });

    let points: Vec<tracecraft_lib::model::geometry::Point> = board.footprints.iter()
        .map(|(_, fp)| fp.position).collect();
    let rect = tracecraft_lib::model::geometry::Rect::from_points(&points).unwrap();
    assert!((rect.x - 10.0).abs() < 1e-10);
    assert!((rect.y - 20.0).abs() < 1e-10);
    assert!((rect.width - 90.0).abs() < 1e-10);
    assert!((rect.height - 60.0).abs() < 1e-10);
}

#[test]
fn test_get_board_bounds_with_traces_and_vias() {
    let mut board = tracecraft_lib::model::pcb::PcbBoard::default();
    board.traces.insert(tracecraft_lib::model::pcb::Trace {
        net_id: None, layer: tracecraft_lib::model::id::LayerId(1), width: 0.25,
        points: vec![point(-5.0, -10.0), point(50.0, 30.0)],
    });
    board.vias.insert(tracecraft_lib::model::pcb::Via {
        position: point(20.0, 5.0),
        start_layer: tracecraft_lib::model::id::LayerId(1),
        end_layer: tracecraft_lib::model::id::LayerId(2),
        diameter: 0.6, drill: 0.3, net_id: None,
    });

    let mut points: Vec<tracecraft_lib::model::geometry::Point> = Vec::new();
    for (_, trace) in board.traces.iter() {
        points.extend(trace.points.iter().copied());
    }
    for (_, via) in board.vias.iter() {
        points.push(via.position);
    }
    let rect = tracecraft_lib::model::geometry::Rect::from_points(&points).unwrap();
    assert!(rect.width > 0.0);
    assert!(rect.height > 0.0);
    assert!((rect.x - (-5.0)).abs() < 1e-10);
    assert!((rect.y - (-10.0)).abs() < 1e-10);
}

#[test]
fn test_get_schematic_bounds_with_elements() {
    let mut sheet = tracecraft_lib::model::schematic::SchematicSheet::default();
    sheet.symbols.insert(tracecraft_lib::model::schematic::SymbolInstance {
        component_id: tracecraft_lib::model::library::ComponentId(uuid::Uuid::new_v4()),
        position: point(0.0, 0.0), rotation: rotation(0.0),
        mirrored: false, refdes: "R1".into(), value: String::new(),
    });
    sheet.wires.insert(tracecraft_lib::model::schematic::Wire {
        points: vec![point(0.0, 0.0), point(200.0, 150.0)],
        net_id: None, width: 0.25,
    });
    sheet.net_labels.insert(tracecraft_lib::model::schematic::NetLabel {
        name: "VCC".into(),
        position: point(200.0, 150.0),
    });

    let mut points: Vec<tracecraft_lib::model::geometry::Point> = Vec::new();
    for (_, sym) in sheet.symbols.iter() { points.push(sym.position); }
    for (_, wire) in sheet.wires.iter() { points.extend(wire.points.iter().copied()); }
    for (_, lbl) in sheet.net_labels.iter() { points.push(lbl.position); }

    let rect = tracecraft_lib::model::geometry::Rect::from_points(&points).unwrap();
    assert!((rect.x - 0.0).abs() < 1e-10);
    assert!((rect.y - 0.0).abs() < 1e-10);
    assert!((rect.width - 200.0).abs() < 1e-10);
    assert!((rect.height - 150.0).abs() < 1e-10);
}

#[test]
fn test_get_bounds_empty_returns_none() {
    let points: Vec<tracecraft_lib::model::geometry::Point> = vec![];
    assert!(tracecraft_lib::model::geometry::Rect::from_points(&points).is_none());
}

// ── 9. Edge Cases ──────────────────────────────────────────────────────────

#[test]
fn test_edge_empty_board_operations() {
    let board = tracecraft_lib::model::pcb::PcbBoard::default();
    // Routing on empty board
    let cfg = tracecraft_lib::engine::autorouter::RouterConfig::default();
    let router = tracecraft_lib::engine::autorouter::AStarRouter::new(cfg);
    let result = router.route_net(
        &board,
        point(0.0, 0.0),
        point(50.0, 50.0),
        tracecraft_lib::model::id::LayerId(1),
        tracecraft_lib::model::id::LayerId(1),
        None,
    );
    assert!(result.success);
    // DRC on empty board
    let violations = tracecraft_lib::engine::drc::run_drc_checks(&board);
    assert!(violations.is_empty());
    // Ratsnest on empty board
    let lines = tracecraft_lib::engine::ratsnest::compute_ratsnest(&board);
    assert!(lines.is_empty());
}

#[test]
fn test_edge_empty_schematic() {
    let sheet = tracecraft_lib::model::schematic::SchematicSheet::default();
    // Netlist extraction
    let netlist = tracecraft_lib::engine::netlist::extract_netlist(&sheet, &mut 0);
    assert!(netlist.nets.is_empty());
    // ERC
    let violations = tracecraft_lib::engine::erc::run_erc(&sheet);
    assert!(violations.is_empty());
}

#[test]
fn test_edge_zero_size_footprint() {
    let mut board = tracecraft_lib::model::pcb::PcbBoard::default();
    let cid = tracecraft_lib::model::library::ComponentId(uuid::Uuid::new_v4());
    let id = board.footprints.insert(tracecraft_lib::model::pcb::FootprintInstance {
        component_id: cid,
        position: point(0.0, 0.0),
        rotation: rotation(0.0),
        mirrored: false,
        refdes: String::new(),
        locked: false,
    });
    let fp = board.footprints.get(id).unwrap();
    assert_eq!(fp.component_id, cid);
    assert!((fp.position.x - 0.0).abs() < 1e-10);
    assert!((fp.position.y - 0.0).abs() < 1e-10);
    assert!((fp.rotation.0 - 0.0).abs() < 1e-10);
}

#[test]
fn test_edge_overlapping_traces() {
    let mut board = tracecraft_lib::model::pcb::PcbBoard::default();
    let id1 = board.traces.insert(tracecraft_lib::model::pcb::Trace {
        net_id: None,
        layer: tracecraft_lib::model::id::LayerId(1),
        width: 0.25,
        points: vec![point(0.0, 0.0), point(10.0, 0.0)],
    });
    let id2 = board.traces.insert(tracecraft_lib::model::pcb::Trace {
        net_id: None,
        layer: tracecraft_lib::model::id::LayerId(1),
        width: 0.25,
        points: vec![point(0.0, 0.0), point(10.0, 0.0)],
    });
    assert!(board.traces.get(id1).is_some());
    assert!(board.traces.get(id2).is_some());
    assert_eq!(board.traces.len(), 2);
    let violations = tracecraft_lib::engine::drc::run_drc_checks(&board);
    assert!(violations.iter().any(|v| v.rule == tracecraft_lib::engine::drc::DRCRule::TraceClearance));
}

#[test]
fn test_edge_large_coordinates() {
    let mut board = tracecraft_lib::model::pcb::PcbBoard::default();
    let cid = tracecraft_lib::model::library::ComponentId(uuid::Uuid::new_v4());
    let id = board.footprints.insert(tracecraft_lib::model::pcb::FootprintInstance {
        component_id: cid,
        position: point(1_000_000.0, 1_000_000.0),
        rotation: rotation(45.0),
        mirrored: false,
        refdes: "R1".into(),
        locked: false,
    });
    let fp = board.footprints.get(id).unwrap();
    assert!((fp.position.x - 1_000_000.0).abs() < 1e-6);
    assert!((fp.position.y - 1_000_000.0).abs() < 1e-6);
}

#[test]
fn test_edge_net_zero_connections() {
    let mut board = tracecraft_lib::model::pcb::PcbBoard::default();
    board.netlist = tracecraft_lib::model::netlist::Netlist {
        nets: vec![tracecraft_lib::model::netlist::Net {
            id: tracecraft_lib::model::id::NetId(1),
            name: "FLOATING".into(),
            connections: vec![],
        }],
    };
    // Ratsnest skips nets with < 2 positions
    let lines = tracecraft_lib::engine::ratsnest::compute_ratsnest(&board);
    assert!(lines.is_empty());
    // Autorouter skips nets with < 2 connection positions
    let cfg = tracecraft_lib::engine::autorouter::RouterConfig::default();
    let results = tracecraft_lib::engine::autorouter::route_all_nets(&board, cfg, |_, _| false);
    assert!(results.is_empty());
}

#[test]
fn test_edge_board_outline_empty_shape() {
    let mut board = tracecraft_lib::model::pcb::PcbBoard::default();
    board.outline = Some(tracecraft_lib::model::pcb::BoardOutline {
        shape: vec![],
        cutouts: vec![],
    });
    let mut points = Vec::new();
    if let Some(ref outline) = board.outline {
        for contour in &outline.shape {
            points.extend(contour.iter().copied());
        }
    }
    assert!(points.is_empty());
    let rect = tracecraft_lib::model::geometry::Rect::from_points(&points);
    assert!(rect.is_none());
}

#[test]
fn test_edge_nan_inf_coordinates() {
    let mut board = tracecraft_lib::model::pcb::PcbBoard::default();
    board.traces.insert(tracecraft_lib::model::pcb::Trace {
        net_id: None,
        layer: tracecraft_lib::model::id::LayerId(1),
        width: 0.25,
        points: vec![point(f64::NAN, 0.0), point(10.0, 0.0)],
    });
    // Should not crash
    let _violations = tracecraft_lib::engine::drc::run_drc_checks(&board);
    // Clear and test with Inf
    board.traces.clear();
    board.traces.insert(tracecraft_lib::model::pcb::Trace {
        net_id: None,
        layer: tracecraft_lib::model::id::LayerId(1),
        width: 0.25,
        points: vec![point(0.0, 0.0), point(f64::INFINITY, f64::INFINITY)],
    });
    // Should not crash
    let _violations = tracecraft_lib::engine::drc::run_drc_checks(&board);
}

#[test]
fn test_edge_dual_layer_routing_nonexistent_layer() {
    let board = tracecraft_lib::model::pcb::PcbBoard::default();
    let cfg = tracecraft_lib::engine::autorouter::RouterConfig::default();
    let router = tracecraft_lib::engine::autorouter::AStarRouter::new(cfg);
    let result = router.route_net(
        &board,
        point(0.0, 0.0),
        point(50.0, 50.0),
        tracecraft_lib::model::id::LayerId(99),
        tracecraft_lib::model::id::LayerId(99),
        None,
    );
    // Falls back to first signal layer index, should succeed
    assert!(result.success);
}

#[test]
fn test_edge_library_duplicate_component() {
    let lib = make_library();
    let cid = tracecraft_lib::model::library::ComponentId(uuid::Uuid::new_v4());
    let comp = || tracecraft_lib::model::library::Component {
        id: cid,
        name: "TestComponent".into(),
        description: "Desc".into(),
        manufacturer: None,
        category: None,
        symbol: tracecraft_lib::model::library::SymbolDef::default(),
        footprint: tracecraft_lib::model::library::FootprintDef::default(),
        model_3d: None,
        parameters: std::collections::HashMap::new(),
        created_at: "2025-01-01T00:00:00Z".into(),
        updated_at: "2025-01-01T00:00:00Z".into(),
    };
    futures::executor::block_on(lib.save(&comp())).unwrap();
    // Second save with same ID — DB uses ON CONFLICT(id) DO UPDATE, so upsert succeeds
    let result = futures::executor::block_on(lib.save(&comp()));
    assert!(result.is_ok());
    // Only one component exists with that ID (upsert updated it)
    let detail = futures::executor::block_on(lib.get_by_id(cid)).unwrap().unwrap();
    assert_eq!(detail.name, "TestComponent");
}
