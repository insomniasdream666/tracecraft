use crate::model::pcb::PcbBoard;
use crate::model::geometry::Point;
use crate::model::id::NetId;
use crate::engine::autorouter::RouterConfig;

pub struct RouteSuggestion {
    pub points: Vec<Point>,
    pub confidence: f32,
}

pub fn suggest_route(board: &PcbBoard, _net_id: NetId, from: Point, to: Point) -> Vec<RouteSuggestion> {
    let config = RouterConfig::default();
    let mut occupancy = OccupancyGrid::new(board, &config);
    let path = occupancy.find_path(from, to);

    if path.len() >= 2 {
        vec![RouteSuggestion {
            points: path,
            confidence: 0.9,
        }]
    } else {
        Vec::new()
    }
}

use std::collections::BinaryHeap;

struct OccupancyGrid {
    width: usize,
    height: usize,
    blocked: Vec<Vec<bool>>,
    resolution: f64,
    origin_x: f64,
    origin_y: f64,
}

impl OccupancyGrid {
    fn new(board: &PcbBoard, config: &RouterConfig) -> Self {
        let (min_x, max_x, min_y, max_y) = compute_bounds(board);
        let width = ((max_x - min_x) / config.grid_resolution).ceil() as usize + 1;
        let height = ((max_y - min_y) / config.grid_resolution).ceil() as usize + 1;
        let w = width.max(10);
        let h = height.max(10);

        let mut blocked = vec![vec![false; h]; w];

        for (_, trace) in board.traces.iter() {
            for pt in &trace.points {
                let (gx, gy) = grid_xy(pt.x, pt.y, config.grid_resolution, min_x, min_y, w, h);
                for dx in 0..=1 { for dy in 0..=1 {
                    let nx = (gx + dx).min(w - 1);
                    let ny = (gy + dy).min(h - 1);
                    blocked[nx][ny] = true;
                }}
            }
        }

        OccupancyGrid {
            width: w,
            height: h,
            blocked,
            resolution: config.grid_resolution,
            origin_x: min_x,
            origin_y: min_y,
        }
    }

    fn find_path(&mut self, from: Point, to: Point) -> Vec<Point> {
        let (sx, sy) = grid_xy(from.x, from.y, self.resolution, self.origin_x, self.origin_y, self.width, self.height);
        let (ex, ey) = grid_xy(to.x, to.y, self.resolution, self.origin_x, self.origin_y, self.width, self.height);

        let mut g_score = vec![vec![f64::MAX; self.height]; self.width];
        let mut parent = vec![vec![None; self.height]; self.width];
        let mut open = BinaryHeap::new();

        g_score[sx][sy] = 0.0;
        open.push(State { cost: 0.0, x: sx, y: sy });

        while let Some(State { x, y, .. }) = open.pop() {
            if x == ex && y == ey { break; }
            let dirs = [(0,1),(0,-1),(1,0),(-1,0)];
            for (dx, dy) in dirs {
                let nx = x as isize + dx;
                let ny = y as isize + dy;
                if nx < 0 || ny < 0 || nx >= self.width as isize || ny >= self.height as isize { continue; }
                let (nx, ny) = (nx as usize, ny as usize);
                if self.blocked[nx][ny] { continue; }
                let new_g = g_score[x][y] + 1.0;
                if new_g < g_score[nx][ny] {
                    g_score[nx][ny] = new_g;
                    parent[nx][ny] = Some((x, y));
                    let h = ((ex as f64 - nx as f64).powi(2) + (ey as f64 - ny as f64).powi(2)).sqrt();
                    open.push(State { cost: new_g + h, x: nx, y: ny });
                }
            }
        }

        let mut path = Vec::new();
        let mut cur = Some((ex, ey));
        while let Some((cx, cy)) = cur {
            path.push(grid_to_world(cx, cy, self.resolution, self.origin_x, self.origin_y));
            cur = parent[cx][cy];
        }
        path.reverse();
        path
    }
}

#[derive(Debug, Clone)]
struct State {
    cost: f64,
    x: usize,
    y: usize,
}

impl std::cmp::PartialEq for State {
    fn eq(&self, other: &Self) -> bool { self.cost == other.cost && self.x == other.x && self.y == other.y }
}
impl std::cmp::Eq for State {}
impl std::cmp::PartialOrd for State {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}
impl std::cmp::Ord for State {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        other.cost.partial_cmp(&self.cost).unwrap_or(std::cmp::Ordering::Equal)
    }
}

fn grid_xy(px: f64, py: f64, resolution: f64, origin_x: f64, origin_y: f64, width: usize, height: usize) -> (usize, usize) {
    let x = ((px - origin_x) / resolution) as usize;
    let y = ((py - origin_y) / resolution) as usize;
    (x.min(width - 1), y.min(height - 1))
}

fn grid_to_world(x: usize, y: usize, resolution: f64, origin_x: f64, origin_y: f64) -> Point {
    Point { x: x as f64 * resolution + origin_x, y: y as f64 * resolution + origin_y }
}

fn compute_bounds(board: &PcbBoard) -> (f64, f64, f64, f64) {
    let mut min_x = f64::MAX; let mut max_x = f64::MIN;
    let mut min_y = f64::MAX; let mut max_y = f64::MIN;
    for (_, t) in board.traces.iter() {
        for p in &t.points {
            min_x = min_x.min(p.x); max_x = max_x.max(p.x);
            min_y = min_y.min(p.y); max_y = max_y.max(p.y);
        }
    }
    for (_, f) in board.footprints.iter() {
        min_x = min_x.min(f.position.x); max_x = max_x.max(f.position.x);
        min_y = min_y.min(f.position.y); max_y = max_y.max(f.position.y);
    }
    if min_x == f64::MAX { (0.0, 100.0, 0.0, 100.0) } else {
        let margin = 10.0;
        (min_x - margin, max_x + margin, min_y - margin, max_y + margin)
    }
}
