use std::collections::{HashMap, HashSet};
use crate::model::netlist::*;
use crate::model::schematic::*;
use crate::model::geometry::Point;
use crate::model::id::*;
use crate::model::library::{Component, ComponentId, PinDef};
use crate::model::pcb::*;

/// Component definitions keyed by id, used to resolve real pin geometry.
pub type ComponentMap = HashMap<ComponentId, Component>;

/// Two layout points closer than this (in schematic units, ~mm) are considered
/// electrically joined.
const CONNECT_TOL: f64 = 0.5;

/// Extract a full netlist from a schematic sheet by tracing wire connectivity.
///
/// This is the legacy entry point: without component definitions it falls back
/// to attaching a single pin at each symbol origin. Real connectivity comes
/// from [`extract_netlist_with_components`].
pub fn extract_netlist(sheet: &SchematicSheet, next_net_id: &mut usize) -> Netlist {
    extract_netlist_with_components(sheet, next_net_id, &ComponentMap::new())
}

/// Extract a netlist using the real pin geometry of each placed component.
///
/// Connectivity rules (matching how schematics are drawn):
/// - wire endpoint touching another wire endpoint joins the nets
/// - wire endpoint landing on another wire body joins the nets (T junction)
/// - crossing wire bodies join only when a junction marker is present
/// - a pin joins a net when its tip touches any wire in that net
/// - pins placed directly on top of each other join without a wire
pub fn extract_netlist_with_components(
    sheet: &SchematicSheet,
    next_net_id: &mut usize,
    components: &ComponentMap,
) -> Netlist {
    let wires: Vec<(usize, &Wire)> = sheet.wires.iter().collect();
    if wires.is_empty() {
        return Netlist { nets: vec![] };
    }

    let segments: Vec<Vec<(Point, Point)>> = wires
        .iter()
        .map(|(_, wire)| {
            wire.points
                .windows(2)
                .map(|pair| (pair[0], pair[1]))
                .collect()
        })
        .collect();

    // ── Join wires ──────────────────────────────────────────────────────────
    let mut dsu = Dsu::new(wires.len());
    for i in 0..wires.len() {
        for j in (i + 1)..wires.len() {
            if wires_touch(&segments[i], &segments[j], sheet) {
                dsu.union(i, j);
            }
        }
    }

    // ── Resolve pin endpoints ───────────────────────────────────────────────
    let pins = collect_pin_endpoints(sheet, components);

    // Pin directly on top of another pin (no wire between them).
    let mut pin_groups: HashMap<(i64, i64), Vec<usize>> = HashMap::new();
    for (idx, pin) in pins.iter().enumerate() {
        pin_groups.entry(quantize(&pin.position)).or_default().push(idx);
    }

    // Group pins by the wire cluster they touch.
    let mut cluster_pins: HashMap<usize, Vec<usize>> = HashMap::new();
    let mut orphan_pins: Vec<usize> = Vec::new();
    for (idx, pin) in pins.iter().enumerate() {
        let mut cluster = None;
        for (wi, wire_segments) in segments.iter().enumerate() {
            if wire_segments
                .iter()
                .any(|(a, b)| point_segment_distance(&pin.position, a, b) <= CONNECT_TOL)
            {
                cluster = Some(dsu.find(wi));
                break;
            }
        }
        match cluster {
            Some(root) => cluster_pins.entry(root).or_default().push(idx),
            None => orphan_pins.push(idx),
        }
    }

    // Pins that share a location but touch no wire still form a net.
    let mut orphan_nets: Vec<Vec<usize>> = Vec::new();
    let mut seen_orphan: HashSet<usize> = HashSet::new();
    for idx in &orphan_pins {
        if !seen_orphan.insert(*idx) {
            continue;
        }
        let key = quantize(&pins[*idx].position);
        let group = pin_groups.get(&key).cloned().unwrap_or_default();
        let attached: Vec<usize> = group
            .into_iter()
            .filter(|g| orphan_pins.contains(g))
            .collect();
        if attached.len() >= 2 {
            for g in &attached {
                seen_orphan.insert(*g);
            }
            orphan_nets.push(attached);
        }
    }

    // ── Emit nets ───────────────────────────────────────────────────────────
    let mut nets: Vec<Net> = Vec::new();
    let mut ordered_roots: Vec<usize> = Vec::new();
    for wi in 0..wires.len() {
        let root = dsu.find(wi);
        if !ordered_roots.contains(&root) {
            ordered_roots.push(root);
        }
    }

    let mut emitted: HashSet<String> = HashSet::new();
    for root in ordered_roots {
        let wire_indices: Vec<usize> = (0..wires.len())
            .filter(|wi| dsu.find(*wi) == root)
            .collect();
        let pin_indices = cluster_pins.get(&root).cloned().unwrap_or_default();
        let name = resolve_net_name(sheet, &wires, &wire_indices, &pins, &pin_indices, next_net_id);
        let connections = pin_connections(&pins, &pin_indices);
        let key = format!("{}|{:?}", name, connections);
        if !emitted.insert(key) {
            continue;
        }
        nets.push(Net {
            id: NetId(*next_net_id),
            name,
            connections,
        });
        *next_net_id += 1;
    }

    for group in orphan_nets {
        let name = resolve_net_name(sheet, &wires, &[], &pins, &group, next_net_id);
        let connections = pin_connections(&pins, &group);
        nets.push(Net {
            id: NetId(*next_net_id),
            name,
            connections,
        });
        *next_net_id += 1;
    }

    Netlist { nets }
}

// ── Helpers ────────────────────────────────────────────────────────────────

#[derive(Debug, Clone)]
struct PinEndpoint {
    symbol: usize,
    refdes: String,
    number: String,
    is_ground: bool,
    is_power: bool,
    position: Point,
}

fn collect_pin_endpoints(sheet: &SchematicSheet, components: &ComponentMap) -> Vec<PinEndpoint> {
    let mut out = Vec::new();
    for (sym_id, sym) in sheet.symbols.iter() {
        let refdes = if sym.refdes.is_empty() {
            format!("U{}", sym_id + 1)
        } else {
            sym.refdes.clone()
        };
        match components.get(&sym.component_id) {
            Some(component) if !component.symbol.pins.is_empty() => {
                for pin in &component.symbol.pins {
                    out.push(pin_endpoint(sym_id, &refdes, pin, sym));
                }
            }
            _ => {
                // Fallback for projects saved before component pin data was
                // available: treat the symbol origin as pin 1.
                out.push(PinEndpoint {
                    symbol: sym_id,
                    refdes,
                    number: "1".to_string(),
                    is_ground: false,
                    is_power: false,
                    position: sym.position,
                });
            }
        }
    }
    out
}

fn pin_endpoint(sym_id: usize, refdes: &str, pin: &PinDef, sym: &SymbolInstance) -> PinEndpoint {
    PinEndpoint {
        symbol: sym_id,
        refdes: refdes.to_string(),
        number: pin.number.clone(),
        is_ground: matches!(pin.electrical_type, crate::model::library::PinType::Gnd),
        is_power: matches!(pin.electrical_type, crate::model::library::PinType::Power),
        position: transform_pin(sym, &pin.position),
    }
}

fn transform_pin(sym: &SymbolInstance, pin: &Point) -> Point {
    let x = if sym.mirrored { -pin.x } else { pin.x };
    let y = pin.y;
    let angle = sym.rotation.0.to_radians();
    let (sin, cos) = angle.sin_cos();
    Point::new(
        sym.position.x + x * cos - y * sin,
        sym.position.y + x * sin + y * cos,
    )
}

fn pin_connections(pins: &[PinEndpoint], indices: &[usize]) -> Vec<PinRef> {
    let mut connections: Vec<PinRef> = indices
        .iter()
        .map(|idx| PinRef {
            refdes: pins[*idx].refdes.clone(),
            pin_number: pins[*idx].number.clone(),
        })
        .collect();
    connections.sort_by(|a, b| a.refdes.cmp(&b.refdes).then(a.pin_number.cmp(&b.pin_number)));
    connections.dedup();
    connections
}

fn resolve_net_name(
    sheet: &SchematicSheet,
    wires: &[(usize, &Wire)],
    wire_indices: &[usize],
    pins: &[PinEndpoint],
    pin_indices: &[usize],
    next_net_id: &mut usize,
) -> String {
    // 1. Explicit label touching one of the wires in this net.
    for &wi in wire_indices {
        let wire = wires[wi].1;
        for (_, label) in sheet.net_labels.iter() {
            let on_wire = wire
                .points
                .windows(2)
                .any(|pair| point_segment_distance(&label.position, &pair[0], &pair[1]) <= CONNECT_TOL)
                || wire
                    .points
                    .iter()
                    .any(|p| p.distance(&label.position) <= CONNECT_TOL);
            if on_wire {
                return label.name.clone();
            }
        }
    }

    // 2. A ground pin names the net GND.
    if pin_indices.iter().any(|idx| pins[*idx].is_ground) {
        return "GND".to_string();
    }

    // 3. Power symbols (VCC, +5V, ...) name the net after the symbol value.
    for &pi in pin_indices {
        let pin = &pins[pi];
        if !pin.is_power {
            continue;
        }
        if let Some((_, sym)) = sheet.symbols.iter().find(|(id, _)| *id == pin.symbol) {
            let value = sym.value.trim();
            if !value.is_empty() && !value.eq_ignore_ascii_case("power") {
                return value.to_string();
            }
        }
    }

    format!("N${}", *next_net_id)
}

fn wires_touch(a: &[(Point, Point)], b: &[(Point, Point)], sheet: &SchematicSheet) -> bool {
    // Shared or nearly-shared endpoints.
    for (a1, a2) in a {
        for (b1, b2) in b {
            if a1.distance(b1) <= CONNECT_TOL
                || a1.distance(b2) <= CONNECT_TOL
                || a2.distance(b1) <= CONNECT_TOL
                || a2.distance(b2) <= CONNECT_TOL
            {
                return true;
            }
            // T junction: an endpoint of one wire lands on the body of the other.
            if point_segment_distance(a1, b1, b2) <= CONNECT_TOL
                || point_segment_distance(a2, b1, b2) <= CONNECT_TOL
                || point_segment_distance(b1, a1, a2) <= CONNECT_TOL
                || point_segment_distance(b2, a1, a2) <= CONNECT_TOL
            {
                return true;
            }
        }
    }

    // Crossing bodies connect only through an explicit junction dot.
    for (_, junction) in sheet.junctions.iter() {
        let on_a = a
            .iter()
            .any(|(p, q)| point_segment_distance(&junction.position, p, q) <= CONNECT_TOL);
        let on_b = b
            .iter()
            .any(|(p, q)| point_segment_distance(&junction.position, p, q) <= CONNECT_TOL);
        if on_a && on_b {
            return true;
        }
    }

    false
}

fn point_segment_distance(p: &Point, a: &Point, b: &Point) -> f64 {
    let dx = b.x - a.x;
    let dy = b.y - a.y;
    let len_sq = dx * dx + dy * dy;
    if len_sq <= f64::EPSILON {
        return p.distance(a);
    }
    let t = (((p.x - a.x) * dx + (p.y - a.y) * dy) / len_sq).clamp(0.0, 1.0);
    let proj = Point::new(a.x + t * dx, a.y + t * dy);
    p.distance(&proj)
}

fn quantize(p: &Point) -> (i64, i64) {
    // Round to 0.1 unit buckets so floating point noise does not split a node.
    let scale = 10.0;
    ((p.x * scale).round() as i64, (p.y * scale).round() as i64)
}

/// Connect relevant pins from symbols to footprints based on refdes
pub fn map_netlist_to_board(netlist: &Netlist, board: &PcbBoard) -> Netlist {
    let mut mapped_nets = Vec::new();

    for net in &netlist.nets {
        let mut mapped_connections = Vec::new();

        // Map schematic refdes/pins to PCB footprint positions
        for conn in &net.connections {
            let _has_footprint = board.footprints.iter().any(|(_, fp)| fp.refdes == conn.refdes);
            mapped_connections.push(conn.clone());
        }

        mapped_nets.push(Net {
            id: net.id,
            name: net.name.clone(),
            connections: mapped_connections,
        });
    }

    Netlist { nets: mapped_nets }
}

/// Compute diff between old and new netlist
pub fn diff_netlists(old: &Netlist, new: &Netlist) -> NetlistDiff {
    let added: Vec<_> = new.nets.iter()
        .filter(|n| !old.nets.iter().any(|on| on.id == n.id))
        .cloned()
        .collect();
    let removed: Vec<_> = old.nets.iter()
        .filter(|n| !new.nets.iter().any(|nn| nn.id == n.id))
        .cloned()
        .collect();

    let mut modified = Vec::new();
    for nn in &new.nets {
        if let Some(on) = old.nets.iter().find(|on| on.id == nn.id) {
            if on.connections != nn.connections || on.name != nn.name {
                modified.push((nn.name.clone(), on.clone(), nn.clone()));
            }
        }
    }

    NetlistDiff { added, removed, modified }
}

// ── Union-find over wire indices ───────────────────────────────────────────

struct Dsu {
    parent: Vec<usize>,
    rank: Vec<u8>,
}

impl Dsu {
    fn new(n: usize) -> Self {
        Self {
            parent: (0..n).collect(),
            rank: vec![0; n],
        }
    }

    fn find(&mut self, x: usize) -> usize {
        if self.parent[x] != x {
            let root = self.find(self.parent[x]);
            self.parent[x] = root;
        }
        self.parent[x]
    }

    fn union(&mut self, a: usize, b: usize) {
        let (ra, rb) = (self.find(a), self.find(b));
        if ra == rb {
            return;
        }
        if self.rank[ra] < self.rank[rb] {
            self.parent[ra] = rb;
        } else if self.rank[ra] > self.rank[rb] {
            self.parent[rb] = ra;
        } else {
            self.parent[rb] = ra;
            self.rank[ra] += 1;
        }
    }
}
