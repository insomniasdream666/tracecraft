use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use crate::model::pcb::*;
use crate::model::geometry::*;
use crate::model::id::*;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RatsnestLine {
    pub net_id: Option<NetId>,
    pub start: Point,
    pub end: Point,
    pub connected: bool,
}

/// Compute MST ratsnest lines using the board's netlist.
/// For each net, collects all footprint positions (matched by refdes),
/// computes a minimum spanning tree, and marks edges that are already routed.
pub fn compute_ratsnest(board: &PcbBoard) -> Vec<RatsnestLine> {
    let mut lines = Vec::new();
    let netlist = &board.netlist;

    // Build refdes → position lookup
    let mut fp_positions: HashMap<&str, Point> = HashMap::new();
    for (_, fp) in board.footprints.iter() {
        fp_positions.insert(fp.refdes.as_str(), fp.position);
    }

    for net in &netlist.nets {
        // Collect positions of all footprints referenced by this net
        let mut positions = Vec::new();
        for conn in &net.connections {
            if let Some(&pos) = fp_positions.get(conn.refdes.as_str()) {
                positions.push((conn.refdes.clone(), pos));
            }
        }

        if positions.len() < 2 {
            continue;
        }

        // MST on positions (ignoring refdes, just using position)
        let mst_edges = prim_mst(&positions);

        for (i, j) in &mst_edges {
            let start = positions[*i].1;
            let end = positions[*j].1;
            let already_routed = trace_exists_between(board, &start, &end);

            lines.push(RatsnestLine {
                net_id: Some(net.id),
                start,
                end,
                connected: already_routed,
            });
        }
    }

    lines
}

fn prim_mst(nodes: &[(String, Point)]) -> Vec<(usize, usize)> {
    let n = nodes.len();
    if n < 2 {
        return vec![];
    }

    let mut in_tree = vec![false; n];
    let mut min_dist = vec![f64::INFINITY; n];
    let mut parent = vec![0usize; n];
    let mut edges = Vec::new();

    min_dist[0] = 0.0;

    for _ in 0..n {
        let u = (0..n)
            .filter(|&i| !in_tree[i])
            .min_by(|&i, &j| min_dist[i].partial_cmp(&min_dist[j]).unwrap_or(std::cmp::Ordering::Greater))
            .unwrap_or(0);

        in_tree[u] = true;

        if u != 0 {
            edges.push((parent[u], u));
        }

        for v in 0..n {
            if !in_tree[v] {
                let dist = nodes[u].1.distance(&nodes[v].1);
                if dist < min_dist[v] {
                    min_dist[v] = dist;
                    parent[v] = u;
                }
            }
        }
    }

    edges
}

fn trace_exists_between(board: &PcbBoard, a: &Point, b: &Point) -> bool {
    let tolerance = 1.0;
    for (_, trace) in board.traces.iter() {
        if trace.points.len() < 2 {
            continue;
        }
        let first = &trace.points[0];
        let last = &trace.points[trace.points.len() - 1];
        if (first.distance(a) < tolerance && last.distance(b) < tolerance)
            || (first.distance(b) < tolerance && last.distance(a) < tolerance)
        {
            return true;
        }
    }
    false
}
