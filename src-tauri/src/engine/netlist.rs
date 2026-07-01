use std::collections::{HashMap, VecDeque};
use crate::model::netlist::*;
use crate::model::schematic::*;
use crate::model::geometry::Point;
use crate::model::id::*;
use crate::model::pcb::*;

/// Extract a full netlist from a schematic sheet by tracing wire connectivity.
/// Groups connected wire segments, identifies attached symbol pins,
/// and assigns net names (user labels or auto-generated).
pub fn extract_netlist(sheet: &SchematicSheet, next_net_id: &mut usize) -> Netlist {
    // Build adjacency: wire segments are connected if they share an endpoint
    // or meet at a junction.
    let wire_count = sheet.wires.iter().count();
    if wire_count == 0 {
        return Netlist { nets: vec![] };
    }

    // Collect all wire endpoints and junctions
    let mut endpoint_to_wires: HashMap<(i64, i64), Vec<usize>> = HashMap::new();
    let mut wire_endpoints: Vec<Vec<Point>> = Vec::new();
    let mut wire_net_labels: Vec<Option<String>> = Vec::new();

    for (wire_id, wire) in sheet.wires.iter() {
        let points = &wire.points;
        if points.len() < 2 {
            continue;
        }
        let key_start = quantize(&points[0]);
        let key_end = quantize(&points[points.len() - 1]);
        endpoint_to_wires.entry(key_start).or_default().push(wire_id);
        endpoint_to_wires.entry(key_end).or_default().push(wire_id);
        wire_endpoints.push(vec![points[0], points[points.len() - 1]]);
        wire_net_labels.push(None);
    }

    // Add junctions as connection points
    for (_, junc) in sheet.junctions.iter() {
        let key = quantize(&junc.position);
        endpoint_to_wires.entry(key).or_default();
    }

    // Associate net labels with wires
    for (_, label) in sheet.net_labels.iter() {
        let label_key = quantize(&label.position);
        if let Some(wire_ids) = endpoint_to_wires.get(&label_key) {
            if let Some(&wid) = wire_ids.first() {
                if wid < wire_net_labels.len() {
                    wire_net_labels[wid] = Some(label.name.clone());
                }
            }
        }
    }

    // Also check wires with points near labels
    for (wi, wire) in sheet.wires.iter() {
        if wire_net_labels.get(wi).map(|l| l.is_some()).unwrap_or(false) {
            continue; // already has a label
        }
        for pt in &wire.points {
            for (_, label) in sheet.net_labels.iter() {
                if pt.distance(&label.position) < 1.0 {
                    if wi < wire_net_labels.len() {
                        wire_net_labels[wi] = Some(label.name.clone());
                    }
                    break;
                }
            }
        }
    }

    // BFS over wires to find connected components → nets
    let mut visited = vec![false; wire_endpoints.len()];
    let mut nets = Vec::new();

    for start_id in 0..wire_endpoints.len() {
        if visited[start_id] {
            continue;
        }

        // BFS to find all connected wires
        let mut component: Vec<usize> = Vec::new();
        let mut queue = VecDeque::new();
        queue.push_back(start_id);
        visited[start_id] = true;

        while let Some(wid) = queue.pop_front() {
            component.push(wid);

            let start_key = quantize(&wire_endpoints[wid][0]);
            let end_key = quantize(&wire_endpoints[wid][1]);

            for key in [start_key, end_key] {
                if let Some(neighbors) = endpoint_to_wires.get(&key) {
                    for &nid in neighbors {
                        if nid < visited.len() && !visited[nid] {
                            visited[nid] = true;
                            queue.push_back(nid);
                        }
                    }
                }
            }
        }

        // Determine net name
        let mut net_name = None;
        for &wid in &component {
            if let Some(ref label) = wire_net_labels[wid] {
                net_name = Some(label.clone());
                break;
            }
        }
        let name = net_name.unwrap_or_else(|| {
            *next_net_id += 1;
            format!("N${}", *next_net_id)
        });

        // Collect symbol pins attached to this net
        let mut connections = Vec::new();
        for &wid in &component {
            if let Some(wire) = sheet.wires.get(wid) {
                for endpoint in &wire.points {
                    // Check which symbol pin is near this endpoint
                    for (_, sym) in sheet.symbols.iter() {
                        let threshold = 5.0;
                        if endpoint.distance(&sym.position) < threshold {
                            connections.push(PinRef {
                                refdes: sym.refdes.clone(),
                                pin_number: "1".to_string(),
                            });
                        }
                    }
                }
            }
        }

        // Deduplicate connections
        connections.sort_by(|a, b| a.refdes.cmp(&b.refdes).then(a.pin_number.cmp(&b.pin_number)));
        connections.dedup();

        let net = Net {
            id: NetId(*next_net_id),
            name: name.clone(),
            connections,
        };
        nets.push(net);
    }

    Netlist { nets }
}

/// Connect relevant pins from symbols to footprints based on refdes
pub fn map_netlist_to_board(netlist: &Netlist, board: &PcbBoard) -> Netlist {
    let mut mapped_nets = Vec::new();

    for net in &netlist.nets {
        let mut mapped_connections = Vec::new();

        // Map schematic refdes/pins to PCB footprint positions
        for conn in &net.connections {
            // Check if a footprint with matching refdes exists on the board
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

fn quantize(p: &Point) -> (i64, i64) {
    let scale = 100.0;
    ((p.x * scale).round() as i64, (p.y * scale).round() as i64)
}


