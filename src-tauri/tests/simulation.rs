use std::collections::HashMap;
use std::process::Command;

use tracecraft_lib::engine::netlist::{extract_netlist_with_components, ComponentMap};
use tracecraft_lib::io::spice::generate_spice;
use tracecraft_lib::model::geometry::{Point, Rotation};
use tracecraft_lib::model::id::ComponentId;
use tracecraft_lib::model::library::{
    Component, FootprintDef, PinDef, PinType, SymbolDef,
};
use tracecraft_lib::model::schematic::{SchematicSheet, SymbolInstance, Wire};

fn pin(number: &str, name: &str, x: f64, y: f64, kind: PinType) -> PinDef {
    PinDef {
        name: name.to_string(),
        number: number.to_string(),
        position: Point::new(x, y),
        length: 2.54,
        electrical_type: kind,
    }
}

fn component(
    name: &str,
    params: &[(&str, &str)],
    pins: Vec<PinDef>,
) -> Component {
    Component {
        id: ComponentId(uuid::Uuid::new_v4()),
        name: name.to_string(),
        description: String::new(),
        manufacturer: None,
        category: None,
        symbol: SymbolDef {
            pins,
            body: vec![],
            origin: Point::new(0.0, 0.0),
        },
        footprint: FootprintDef::default(),
        model_3d: None,
        parameters: params
            .iter()
            .map(|(k, v)| (k.to_string(), v.to_string()))
            .collect(),
        created_at: "2026-01-01T00:00:00Z".to_string(),
        updated_at: "2026-01-01T00:00:00Z".to_string(),
    }
}

struct Fixture {
    sheet: SchematicSheet,
    components: ComponentMap,
}

/// VCC -- R -- node -- C -- GND, laid out with real pin geometry.
fn fixture() -> Fixture {
    let mut sheet = SchematicSheet::default();
    let mut components = ComponentMap::new();

    let vcc = component(
        "VCC",
        &[("Voltage", "5V")],
        vec![pin("1", "1", 0.0, 0.0, PinType::Power)],
    );
    let gnd = component(
        "GND",
        &[],
        vec![pin("1", "1", 0.0, 0.0, PinType::Gnd)],
    );
    let resistor = component(
        "Resistor",
        &[("Resistance", "1k")],
        vec![
            pin("1", "1", -2.54, 0.0, PinType::Passive),
            pin("2", "2", 2.54, 0.0, PinType::Passive),
        ],
    );
    let capacitor = component(
        "Capacitor",
        &[("Capacitance", "100nF")],
        vec![
            pin("1", "1", -2.54, 0.0, PinType::Passive),
            pin("2", "2", 2.54, 0.0, PinType::Passive),
        ],
    );

    let vcc_id = vcc.id;
    let gnd_id = gnd.id;
    let r_id = resistor.id;
    let c_id = capacitor.id;
    components.insert(vcc_id, vcc);
    components.insert(gnd_id, gnd);
    components.insert(r_id, resistor);
    components.insert(c_id, capacitor);

    let mut place = |component_id: ComponentId, position: (f64, f64), refdes: &str, value: &str| {
        sheet.symbols.insert(SymbolInstance {
            component_id,
            position: Point::new(position.0, position.1),
            rotation: Rotation(0.0),
            mirrored: false,
            refdes: refdes.to_string(),
            value: value.to_string(),
        });
    };

    place(vcc_id, (0.0, 0.0), "V1", "VCC");
    place(r_id, (20.0, 0.0), "R1", "Resistor");
    place(c_id, (40.0, 20.0), "C1", "Capacitor");
    place(gnd_id, (40.0, 40.0), "GND1", "GND");

    let mut wire = |points: Vec<(f64, f64)>| {
        sheet.wires.insert(Wire {
            points: points.into_iter().map(|(x, y)| Point::new(x, y)).collect(),
            net_id: None,
            width: 0.25,
        });
    };

    // V1.1 (0,0) -> R1.1 (17.46,0)
    wire(vec![(0.0, 0.0), (17.46, 0.0)]);
    // R1.2 (22.54,0) -> C1.1 (37.46,20)
    wire(vec![(22.54, 0.0), (22.54, 20.0), (37.46, 20.0)]);
    // C1.2 (42.54,20) -> GND1.1 (40,40)
    wire(vec![(42.54, 20.0), (42.54, 40.0), (40.0, 40.0)]);

    Fixture { sheet, components }
}

#[test]
fn netlist_uses_real_pin_numbers() {
    let Fixture { sheet, components } = fixture();
    let mut next = 0;
    let netlist = extract_netlist_with_components(&sheet, &mut next, &components);

    assert_eq!(netlist.nets.len(), 3, "expected VCC, mid-node and GND nets");

    let by_name: HashMap<&str, Vec<String>> = netlist
        .nets
        .iter()
        .map(|n| {
            (
                n.name.as_str(),
                n.connections
                    .iter()
                    .map(|c| format!("{}.{}", c.refdes, c.pin_number))
                    .collect(),
            )
        })
        .collect();

    assert_eq!(by_name["VCC"], vec!["R1.1", "V1.1"]);
    assert_eq!(by_name["GND"], vec!["C1.2", "GND1.1"]);

    let mid = netlist
        .nets
        .iter()
        .find(|n| n.name.starts_with("N$"))
        .expect("auto-named middle net");
    let mut mid_pins: Vec<String> = mid
        .connections
        .iter()
        .map(|c| format!("{}.{}", c.refdes, c.pin_number))
        .collect();
    mid_pins.sort();
    assert_eq!(mid_pins, vec!["C1.1", "R1.2"]);
}

#[test]
fn spice_netlist_is_well_formed() {
    let Fixture { sheet, components } = fixture();
    let netlist = generate_spice(&sheet, &HashMap::new(), &components, Some(".tran 1m 10m"))
        .expect("netlist generation");

    assert!(netlist.contains("V1 VCC 0 DC 5"), "netlist:\n{}", netlist);
    assert!(netlist.contains("R1 VCC "), "netlist:\n{}", netlist);
    assert!(netlist.contains(" 1k"), "netlist:\n{}", netlist);
    assert!(netlist.contains("C1 "), "netlist:\n{}", netlist);
    assert!(netlist.contains(" 100n"), "netlist:\n{}", netlist);
    assert!(netlist.contains(".tran 1m 10m"));
    assert!(netlist.trim_end().ends_with(".end"));
    assert!(!netlist.contains('$'), "node names must not contain '$':\n{}", netlist);
    // No bogus "net alias" device lines.
    assert!(!netlist.lines().any(|l| l.trim() == "VCC VCC"));
}

#[test]
fn ngspice_runs_generated_netlist() {
    if Command::new("ngspice").arg("-v").output().is_err() {
        eprintln!("ngspice not installed; skipping");
        return;
    }

    let Fixture { sheet, components } = fixture();
    let netlist =
        generate_spice(&sheet, &HashMap::new(), &components, Some(".tran 1m 10m")).unwrap();

    let dir = std::env::temp_dir().join("tracecraft-test-sim");
    std::fs::create_dir_all(&dir).unwrap();
    let stamp = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let cir = dir.join(format!("net-{}.cir", stamp));
    let raw = dir.join(format!("net-{}.raw", stamp));
    std::fs::write(&cir, &netlist).unwrap();

    let output = Command::new("ngspice")
        .arg("-b")
        .arg("-r")
        .arg(&raw)
        .arg(&cir)
        .output()
        .expect("run ngspice");

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        output.status.success(),
        "ngspice failed\nstdout:\n{}\nstderr:\n{}",
        stdout,
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(
        !stdout.contains("Error on line"),
        "ngspice rejected the netlist:\n{}\n{}\n{}",
        netlist,
        stdout,
        String::from_utf8_lossy(&output.stderr)
    );

    let raw_text = std::fs::read_to_string(&raw).expect("rawfile written");
    assert!(
        raw_text.contains("Plotname: Transient Analysis"),
        "rawfile:\n{}",
        &raw_text[..raw_text.len().min(800)]
    );
    assert!(raw_text.contains("No. Points:"), "rawfile missing points");
}
