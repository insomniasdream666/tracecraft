use std::collections::{BinaryHeap, HashMap, HashSet};
use crate::model::pcb::*;
use crate::model::geometry::{Point, Rect};
use crate::model::id::*;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Direction {
    Horizontal,
    Vertical,
    Any,
}

#[derive(Debug, Clone)]
pub struct RouterConfig {
    pub grid_resolution: f64,
    pub via_cost: f64,
    pub trace_cost_per_unit: f64,
    pub preferred_directions: Vec<(u8, Direction)>,
    pub clearance: f64,
}

impl Default for RouterConfig {
    fn default() -> Self {
        Self {
            grid_resolution: 0.1,
            via_cost: 30.0,
            trace_cost_per_unit: 1.0,
            preferred_directions: vec![(1, Direction::Horizontal), (2, Direction::Vertical)],
            clearance: 0.15,
        }
    }
}

#[derive(Debug, Clone)]
pub struct RoutingResult {
    pub traces: Vec<NewTrace>,
    pub success: bool,
    pub path_length: f64,
}

#[derive(Debug, Clone)]
pub struct NewTrace {
    pub net_id: Option<NetId>,
    pub layer: LayerId,
    pub points: Vec<Point>,
    pub width: f64,
}

// ─── Grid helpers ──────────────────────────────────────────────────────────

struct GridCell {
    blocked: bool,
    #[allow(dead_code)]
    cost: f64,
}

struct OccupancyGrid {
    width: usize,
    height: usize,
    layers: Vec<Vec<Vec<GridCell>>>,
    resolution: f64,
    origin_x: f64,
    origin_y: f64,
}

impl OccupancyGrid {
    fn new(
        bounds: &Rect,
        signal_layers: &[u8],
        resolution: f64,
    ) -> Self {
        let w = (bounds.width / resolution).ceil() as usize + 2;
        let h = (bounds.height / resolution).ceil() as usize + 2;
        let layers: Vec<Vec<Vec<GridCell>>> = signal_layers
            .iter()
            .map(|_| {
                (0..h)
                    .map(|_| {
                        (0..w)
                            .map(|_| GridCell {
                                blocked: false,
                                cost: 1.0,
                            })
                            .collect()
                    })
                    .collect()
            })
            .collect();
        Self {
            width: w,
            height: h,
            layers,
            resolution,
            origin_x: bounds.x - resolution,
            origin_y: bounds.y - resolution,
        }
    }

    fn world_to_grid(&self, p: &Point) -> (i64, i64) {
        let gx = ((p.x - self.origin_x) / self.resolution).round() as i64;
        let gy = ((p.y - self.origin_y) / self.resolution).round() as i64;
        (gx, gy)
    }

    #[allow(dead_code)]
    fn grid_to_world(&self, gx: i64, gy: i64) -> Point {
        Point::new(
            gx as f64 * self.resolution + self.origin_x,
            gy as f64 * self.resolution + self.origin_y,
        )
    }

    fn in_bounds(&self, gx: i64, gy: i64) -> bool {
        gx >= 0 && gx < self.width as i64 && gy >= 0 && gy < self.height as i64
    }

    fn is_blocked(&self, layer_idx: usize, gx: i64, gy: i64) -> bool {
        if !self.in_bounds(gx, gy) {
            return true;
        }
        self.layers[layer_idx][gy as usize][gx as usize].blocked
    }

    fn block_circle(&mut self, center: &Point, radius: f64) {
        let (cx, cy) = self.world_to_grid(center);
        let r = (radius / self.resolution).ceil() as i64;
        for dy in -r..=r {
            for dx in -r..=r {
                let dist = ((dx * dx + dy * dy) as f64).sqrt();
                if dist <= r as f64 + 0.5 {
                    let gx = cx + dx;
                    let gy = cy + dy;
                    if self.in_bounds(gx, gy) {
                        for layer in &mut self.layers {
                            layer[gy as usize][gx as usize].blocked = true;
                        }
                    }
                }
            }
        }
    }

    fn block_trace(&mut self, trace: &Trace, clearance: f64) {
        let expansion = trace.width / 2.0 + clearance;
        for w in trace.points.windows(2) {
            let start = &w[0];
            let end = &w[1];
            let (sx, sy) = self.world_to_grid(start);
            let (ex, ey) = self.world_to_grid(end);
            let r = (expansion / self.resolution).ceil() as i64;

            let layer_idx = trace.layer.0 as usize;
            if layer_idx < self.layers.len() {
                self.block_line_segment(
                    layer_idx, sx, sy, ex, ey, r,
                );
            }
        }
    }

    fn block_line_segment(
        &mut self,
        layer_idx: usize,
        x0: i64,
        y0: i64,
        x1: i64,
        y1: i64,
        radius: i64,
    ) {
        let dx = (x1 - x0).abs();
        let dy = -(y1 - y0).abs();
        let sx = if x0 < x1 { 1 } else { -1 };
        let sy = if y0 < y1 { 1 } else { -1 };
        let mut err = dx + dy;
        let mut x = x0;
        let mut y = y0;

        loop {
            for dy2 in -radius..=radius {
                for dx2 in -radius..=radius {
                    let gx = x + dx2;
                    let gy = y + dy2;
                    if self.in_bounds(gx, gy) {
                        self.layers[layer_idx][gy as usize][gx as usize].blocked = true;
                    }
                }
            }
            if x == x1 && y == y1 {
                break;
            }
            let e2 = 2 * err;
            if e2 >= dy {
                err += dy;
                x += sx;
            }
            if e2 <= dx {
                err += dx;
                y += sy;
            }
        }
    }
}

// ─── A* Node ───────────────────────────────────────────────────────────────

#[derive(Debug, Clone, PartialEq)]
struct Node {
    gx: i64,
    gy: i64,
    layer_idx: usize,
    g: f64,
    f: f64,
    prev_gx: i64,
    prev_gy: i64,
    prev_layer: usize,
}

impl Eq for Node {}

impl std::cmp::Ord for Node {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        other.f.partial_cmp(&self.f).unwrap_or(std::cmp::Ordering::Equal)
    }
}

impl std::cmp::PartialOrd for Node {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

// ─── A* Router ─────────────────────────────────────────────────────────────

pub struct AStarRouter {
    pub config: RouterConfig,
}

impl AStarRouter {
    pub fn new(config: RouterConfig) -> Self {
        Self { config }
    }

    /// Route a single net from `start` to `end`. `start_layer` and `end_layer`
    /// determine which signal layers the route can begin/end on.
    pub fn route_net(
        &self,
        board: &PcbBoard,
        start: Point,
        end: Point,
        start_layer: LayerId,
        end_layer: LayerId,
        net_id: Option<NetId>,
    ) -> RoutingResult {
        let cfg = &self.config;
        let signal_layers: Vec<u8> = board
            .layers
            .layers
            .iter()
            .filter(|l| matches!(l.kind, LayerKind::Top | LayerKind::Bottom | LayerKind::Inner))
            .map(|l| l.id.0)
            .collect();

        if signal_layers.is_empty() {
            return RoutingResult {
                traces: vec![],
                success: false,
                path_length: 0.0,
            };
        }

        // Compute board bounds
        let bounds = self.compute_bounds(board);
        let mut grid = OccupancyGrid::new(&bounds, &signal_layers, cfg.grid_resolution);

        // Mark obstacles: footprints (pads), existing traces, vias
        self.mark_obstacles(board, &mut grid, cfg.clearance);

        // Find start/end grid positions
        let (sx, sy) = grid.world_to_grid(&start);
        let (ex, ey) = grid.world_to_grid(&end);

        let start_layer_idx = signal_layers
            .iter()
            .position(|l| *l == start_layer.0)
            .unwrap_or(0);
        let end_layer_idx = signal_layers
            .iter()
            .position(|l| *l == end_layer.0)
            .unwrap_or(0);

        // A* search
        let path = self.astar_search(
            &mut grid,
            sx, sy, start_layer_idx,
            ex, ey, end_layer_idx,
            &signal_layers,
        );

        match path {
            Some(nodes) => {
                let traces = self.path_to_traces(&nodes, &signal_layers, &bounds, cfg);
                let len = traces.iter().flat_map(|t| t.points.windows(2)).map(|w| w[0].distance(&w[1])).sum();
                let traces_with_net: Vec<NewTrace> = traces
                    .into_iter()
                    .map(|t| NewTrace {
                        net_id,
                        layer: t.layer,
                        points: t.points,
                        width: board.design_rules.default_trace_width,
                    })
                    .collect();
                RoutingResult {
                    traces: traces_with_net,
                    success: true,
                    path_length: len,
                }
            }
            None => RoutingResult {
                traces: vec![],
                success: false,
                path_length: 0.0,
            },
        }
    }

    fn astar_search(
        &self,
        grid: &mut OccupancyGrid,
        sx: i64, sy: i64, start_layer: usize,
        ex: i64, ey: i64, end_layer: usize,
        signal_layers: &[u8],
    ) -> Option<Vec<(i64, i64, usize)>> {
        let cfg = &self.config;

        let mut open = BinaryHeap::new();
        let mut open_set: HashSet<(i64, i64, usize)> = HashSet::new();
        let mut closed: HashMap<(i64, i64, usize), Node> = HashMap::new();

        let h_start = self.octile_distance(sx, sy, ex, ey);
        let start_node = Node {
            gx: sx,
            gy: sy,
            layer_idx: start_layer,
            g: 0.0,
            f: h_start,
            prev_gx: sx,
            prev_gy: sy,
            prev_layer: start_layer,
        };
        open.push(start_node.clone());
        open_set.insert((sx, sy, start_layer));
        closed.insert((sx, sy, start_layer), start_node);

        // 8-connected directions plus via transitions
        let dirs: [(i64, i64); 8] = [
            (1, 0), (-1, 0), (0, 1), (0, -1),
            (1, 1), (-1, 1), (1, -1), (-1, -1),
        ];

        while let Some(current) = open.pop() {
            let key = (current.gx, current.gy, current.layer_idx);
            if current.g > closed.get(&key).map(|n| n.g).unwrap_or(f64::INFINITY) {
                continue;
            }

            // Reached goal
            if current.gx == ex && current.gy == ey && current.layer_idx == end_layer {
                return Some(self.reconstruct_path(&closed, current, signal_layers));
            }

            // Explore neighbors on same layer
            for &(dx, dy) in &dirs {
                let nx = current.gx + dx;
                let ny = current.gy + dy;
                if !grid.in_bounds(nx, ny) || grid.is_blocked(current.layer_idx, nx, ny) {
                    continue;
                }

                let is_diag = dx != 0 && dy != 0;
                let move_cost = if is_diag { 1.414 } else { 1.0 };
                let step = move_cost * cfg.trace_cost_per_unit * cfg.grid_resolution;

                // Direction preference penalty
                let dir_penalty = self.direction_penalty(
                    current.layer_idx,
                    dx, dy,
                    &cfg.preferred_directions,
                );

                let g = current.g + step + dir_penalty;
                let h = self.octile_distance(nx, ny, ex, ey);
                let f = g + h;

                let nk = (nx, ny, current.layer_idx);
                if !closed.contains_key(&nk) || g < closed[&nk].g {
                    let node = Node {
                        gx: nx,
                        gy: ny,
                        layer_idx: current.layer_idx,
                        g,
                        f,
                        prev_gx: current.gx,
                        prev_gy: current.gy,
                        prev_layer: current.layer_idx,
                    };
                    closed.insert(nk, node.clone());
                    if !open_set.contains(&nk) {
                        open.push(node);
                        open_set.insert(nk);
                    }
                }
            }

            // Via transitions: jump to other layers at same (gx, gy)
            if signal_layers.len() > 1 {
                for (li, _) in signal_layers.iter().enumerate() {
                    if li == current.layer_idx {
                        continue;
                    }
                    // Check if we can place a via here (not blocked on either layer)
                    if grid.is_blocked(li, current.gx, current.gy) {
                        continue;
                    }

                    let g = current.g + cfg.via_cost;
                    let h = self.octile_distance(current.gx, current.gy, ex, ey);
                    let f = g + h;

                    let nk = (current.gx, current.gy, li);
                    if !closed.contains_key(&nk) || g < closed[&nk].g {
                        let node = Node {
                            gx: current.gx,
                            gy: current.gy,
                            layer_idx: li,
                            g,
                            f,
                            prev_gx: current.gx,
                            prev_gy: current.gy,
                            prev_layer: current.layer_idx,
                        };
                        closed.insert(nk, node.clone());
                        if !open_set.contains(&nk) {
                            open.push(node);
                            open_set.insert(nk);
                        }
                    }
                }
            }
        }

        None
    }

    fn reconstruct_path(
        &self,
        closed: &HashMap<(i64, i64, usize), Node>,
        goal: Node,
        signal_layers: &[u8],
    ) -> Vec<(i64, i64, usize)> {
        let mut path = Vec::new();
        let mut current = goal;

        loop {
            path.push((current.gx, current.gy, current.layer_idx));
            if current.prev_gx == current.gx
                && current.prev_gy == current.gy
                && current.prev_layer == current.layer_idx
            {
                break;
            }
            let key = (current.prev_gx, current.prev_gy, current.prev_layer);
            match closed.get(&key) {
                Some(prev) => current = prev.clone(),
                None => break,
            }
        }

        path.reverse();
        self.simplify_path(&path, signal_layers)
    }

    /// Remove collinear points and merge same-layer segments.
    fn simplify_path(
        &self,
        path: &[(i64, i64, usize)],
        _signal_layers: &[u8],
    ) -> Vec<(i64, i64, usize)> {
        if path.len() <= 2 {
            return path.to_vec();
        }

        let mut simplified = vec![path[0]];
        for i in 1..path.len() - 1 {
            let prev = path[i - 1];
            let curr = path[i];
            let next = path[i + 1];

            // Always keep layer transitions
            if curr.2 != prev.2 || curr.2 != next.2 {
                simplified.push(curr);
                continue;
            }

            // Check collinearity
            let dx1 = curr.0 - prev.0;
            let dy1 = curr.1 - prev.1;
            let dx2 = next.0 - curr.0;
            let dy2 = next.1 - curr.1;

            if dx1 != dx2 || dy1 != dy2 {
                simplified.push(curr);
            }
        }
        simplified.push(path[path.len() - 1]);

        simplified
    }

    fn path_to_traces(
        &self,
        path: &[(i64, i64, usize)],
        signal_layers: &[u8],
        bounds: &Rect,
        cfg: &RouterConfig,
    ) -> Vec<Trace> {
        if path.is_empty() {
            return vec![];
        }

        let origin_x = bounds.x - cfg.grid_resolution;
        let origin_y = bounds.y - cfg.grid_resolution;

        let mut traces: Vec<Trace> = Vec::new();
        let mut current_layer = path[0].2;
        let mut current_segment = Vec::new();

        for &(gx, gy, li) in path {
            let pt = Point::new(
                gx as f64 * cfg.grid_resolution + origin_x,
                gy as f64 * cfg.grid_resolution + origin_y,
            );

            if li != current_layer {
                // Finish current layer segment
                if current_segment.len() >= 2 {
                    traces.push(Trace {
                        net_id: None,
                        layer: LayerId(signal_layers[current_layer]),
                        width: 0.25,
                        points: current_segment.clone(),
                    });
                }
                // Start new layer (with via position at transition point)
                if let Some(last) = current_segment.last() {
                    traces.push(Trace {
                        net_id: None,
                        layer: LayerId(signal_layers[current_layer]),
                        width: 0.25,
                        points: vec![*last, *last],
                    });
                    current_segment = vec![*last, pt];
                } else {
                    current_segment = vec![pt];
                }
                current_layer = li;
            } else {
                current_segment.push(pt);
            }
        }

        if current_segment.len() >= 2 {
            traces.push(Trace {
                net_id: None,
                layer: LayerId(signal_layers[current_layer]),
                width: 0.25,
                points: current_segment,
            });
        }

        traces
    }

    fn mark_obstacles(&self, board: &PcbBoard, grid: &mut OccupancyGrid, clearance: f64) {
        // Block existing traces
        for (_, trace) in board.traces.iter() {
            grid.block_trace(trace, clearance);
        }

        // Block via positions (as circles)
        for (_, via) in board.vias.iter() {
            grid.block_circle(&via.position, via.diameter / 2.0 + clearance);
        }

        // Block footprint pad areas (approximate as circles)
        for (_, fp) in board.footprints.iter() {
            grid.block_circle(&fp.position, 5.0 + clearance);
        }

        // Block zones
        for (_, zone) in board.zones.iter() {
            for pt in &zone.outline {
                grid.block_circle(pt, clearance);
            }
        }
    }

    fn compute_bounds(&self, board: &PcbBoard) -> Rect {
        let mut points = Vec::new();

        // From footprints
        for (_, fp) in board.footprints.iter() {
            points.push(fp.position);
            points.push(Point::new(fp.position.x + 10.0, fp.position.y + 10.0));
        }

        // From traces
        for (_, trace) in board.traces.iter() {
            for pt in &trace.points {
                points.push(*pt);
            }
        }

        // From board outline
        if let Some(outline) = &board.outline {
            for contour in &outline.shape {
                points.extend(contour.iter());
            }
        }

        if points.is_empty() {
            return Rect {
                x: -50.0,
                y: -50.0,
                width: 100.0,
                height: 100.0,
            };
        }

        Rect::from_points(&points).unwrap_or(Rect {
            x: -50.0,
            y: -50.0,
            width: 100.0,
            height: 100.0,
        })
    }

    fn octile_distance(&self, ax: i64, ay: i64, bx: i64, by: i64) -> f64 {
        let dx = (ax - bx).abs() as f64;
        let dy = (ay - by).abs() as f64;
        let d = dx.max(dy);
        let s = dx.min(dy);
        (s * 1.414 + (d - s)) * self.config.grid_resolution
    }

    fn direction_penalty(
        &self,
        layer_idx: usize,
        dx: i64,
        dy: i64,
        preferred: &[(u8, Direction)],
    ) -> f64 {
        let layer_id = layer_idx as u8 + 1;
        for &(lid, ref dir) in preferred {
            if lid == layer_id {
                match dir {
                    Direction::Horizontal if dy != 0 => return 0.5,
                    Direction::Vertical if dx != 0 => return 0.5,
                    _ => return 0.0,
                }
            }
        }
        0.0
    }
}

// ─── Batch Router ──────────────────────────────────────────────────────────

/// Route all unpouted nets on the board.
/// Batch-route all nets using the board's netlist.
/// Sorts nets by pin count (fewest first), routes each net,
/// and retries failed nets with relaxed clearance.
pub fn route_all_nets<F: Fn(usize, usize) -> bool>(
    board: &PcbBoard,
    config: RouterConfig,
    should_cancel: F,
) -> Vec<(Option<NetId>, RoutingResult)> {
    let router = AStarRouter::new(config.clone());
    let mut results = Vec::new();

    // Build refdes → position map
    let fp_positions: std::collections::HashMap<&str, Point> = board
        .footprints
        .iter()
        .map(|(_, fp)| (fp.refdes.as_str(), fp.position))
        .collect();

    // Collect nets with their footprint positions
    let mut nets_sorted: Vec<(NetId, Vec<Point>)> = board.netlist.nets.iter()
        .filter_map(|net| {
            let points: Vec<Point> = net.connections.iter()
                .filter_map(|conn| fp_positions.get(conn.refdes.as_str()).copied())
                .collect();
            if points.len() >= 2 {
                Some((net.id, points))
            } else {
                None
            }
        })
        .collect();

    // Sort by pin count (fewest first)
    nets_sorted.sort_by_key(|(_, pts)| pts.len());

    let total = nets_sorted.len();
    for (i, (net_id, points)) in nets_sorted.iter().enumerate() {
        if should_cancel(i, total) { break; }
        for pair in points.windows(2) {
            let start = pair[0];
            let end = pair[1];
            let result = router.route_net(board, start, end, LayerId(1), LayerId(2), Some(*net_id));
            results.push((Some(*net_id), result));
        }
    }

    // Retry failed nets with relaxed clearance
    let config_clone = RouterConfig {
        clearance: config.clearance * 0.5,
        preferred_directions: config.preferred_directions.clone(),
        grid_resolution: config.grid_resolution,
        via_cost: config.via_cost,
        trace_cost_per_unit: config.trace_cost_per_unit,
    };
    let relaxed_router = AStarRouter::new(config_clone);

    for (i, (net_id, points)) in nets_sorted.iter().enumerate() {
        if should_cancel(i, total) { break; }
        for pair in points.windows(2) {
            let start = pair[0];
            let end = pair[1];
            let has_failed = results.iter().any(|(id, r)| *id == Some(*net_id) && !r.success);
            if has_failed {
                let result = relaxed_router.route_net(board, start, end, LayerId(1), LayerId(2), Some(*net_id));
                results.push((Some(*net_id), result));
            }
        }
    }

    results
}
