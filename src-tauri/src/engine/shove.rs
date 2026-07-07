use crate::model::geometry::*;
use crate::model::pcb::*;
use crate::model::id::{LayerId, NetId};

/// Resolve conflicts by shoving existing traces out of the way when a new trace
/// would collide with them. Returns the adjusted points for the new trace and
/// modifies the board's existing traces in place.
pub fn resolve_conflicts(
    new_points: &[Point],
    width: f64,
    clearance: f64,
    board: &mut PcbBoard,
    layer: LayerId,
    net_id: Option<NetId>,
) -> Vec<Point> {
    if new_points.len() < 2 {
        return new_points.to_vec();
    }

    let mut adjusted = new_points.to_vec();
    let min_gap = width / 2.0 + clearance;

    // Collect trace IDs and their states so we can modify them
    let trace_ids: Vec<usize> = board.traces.iter()
        .filter(|(_, t)| t.layer == layer)
        .map(|(id, _)| id)
        .collect();

    for tid in &trace_ids {
        let existing = match board.traces.get(*tid) {
            Some(t) => t.clone(),
            None => continue,
        };
        if existing.points.len() < 2 { continue; }
        if existing.net_id == net_id { continue; } // same net, no conflict

        let mut pushed = existing.points.clone();
        let mut any_push = false;

        for i in 0..adjusted.len().saturating_sub(1) {
            let a1 = adjusted[i];
            let a2 = adjusted[i + 1];
            for j in 0..existing.points.len().saturating_sub(1) {
                let b1 = existing.points[j];
                let b2 = existing.points[j + 1];

                if let Some(push) = segment_push_vector(a1, a2, b1, b2, min_gap + existing.width / 2.0) {
                    // Push the existing trace segment
                    if j < pushed.len() {
                        pushed[j].x += push.x;
                        pushed[j].y += push.y;
                    }
                    if j + 1 < pushed.len() {
                        pushed[j + 1].x += push.x;
                        pushed[j + 1].y += push.y;
                    }
                    any_push = true;

                    // Also shove the new trace away slightly for visual clearance
                    if i + 1 < adjusted.len() {
                        let push_back = Point::new(-push.x * 0.3, -push.y * 0.3);
                        adjusted[i + 1].x += push_back.x;
                        adjusted[i + 1].y += push_back.y;
                    }
                }
            }
        }

        if any_push {
            // Smooth the pushed trace to avoid kinks
            let smoothed = smooth_trace(&pushed);
            if let Some(t) = board.traces.get_mut(*tid) {
                t.points = smoothed;
            }
        }
    }

    adjusted
}

/// Calculate a vector to push segment b1-b2 away from segment a1-a2
fn segment_push_vector(
    a1: Point, a2: Point,
    b1: Point, b2: Point,
    min_dist: f64,
) -> Option<Point> {
    let (closest_a, closest_b) = closest_points_on_segments(a1, a2, b1, b2);
    let dx = closest_b.x - closest_a.x;
    let dy = closest_b.y - closest_a.y;
    let dist = (dx * dx + dy * dy).sqrt();

    if dist < min_dist {
        let overlap = if dist < 1e-9 { min_dist } else { min_dist - dist };
        let (nx, ny) = if dist < 1e-9 {
            // Segments intersect - push in a default perpendicular direction
            let bdx = b2.x - b1.x;
            let bdy = b2.y - b1.y;
            let blen = (bdx * bdx + bdy * bdy).sqrt();
            if blen < 1e-9 { return None; }
            (-bdy / blen, bdx / blen)
        } else {
            // Push perpendicular to the existing trace's direction, away from the new trace
            let bdx = b2.x - b1.x;
            let bdy = b2.y - b1.y;
            let blen = (bdx * bdx + bdy * bdy).sqrt();
            if blen < 1e-9 { return None; }
            let nx = -bdy / blen;
            let ny = bdx / blen;
            let dot = nx * dx + ny * dy;
            if dot >= 0.0 { (nx, ny) } else { (-nx, -ny) }
        };
        return Some(Point::new(nx * overlap, ny * overlap));
    }
    None
}

/// Find closest points on two line segments
fn closest_points_on_segments(
    a1: Point, a2: Point,
    b1: Point, b2: Point,
) -> (Point, Point) {
    // Simple approach: sample points along segments and find minimum distance
    let mut best_a = a1;
    let mut best_b = b1;
    let mut best_d = f64::MAX;

    let samples = 10;
    for ai in 0..=samples {
        let t = ai as f64 / samples as f64;
        let pa = Point::new(a1.x + (a2.x - a1.x) * t, a1.y + (a2.y - a1.y) * t);
        for bi in 0..=samples {
            let s = bi as f64 / samples as f64;
            let pb = Point::new(b1.x + (b2.x - b1.x) * s, b1.y + (b2.y - b1.y) * s);
            let d = pa.distance(&pb);
            if d < best_d {
                best_d = d;
                best_a = pa;
                best_b = pb;
            }
        }
    }
    (best_a, best_b)
}

/// Remove collinear points and smooth a trace
fn smooth_trace(points: &[Point]) -> Vec<Point> {
    if points.len() < 3 { return points.to_vec(); }

    let mut result = Vec::new();
    result.push(points[0]);

    for i in 1..points.len() - 1 {
        let prev = &result[result.len() - 1];
        let curr = &points[i];
        let next = &points[i + 1];

        // Check if curr is collinear with prev-next line
        let cross = (curr.x - prev.x) * (next.y - curr.y)
                  - (curr.y - prev.y) * (next.x - curr.x);
        if cross.abs() > 0.5 {
            result.push(*curr);
        }
    }
    result.push(points[points.len() - 1]);
    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_no_conflict() {
        let mut board = PcbBoard::default();
        let points = vec![Point::new(0.0, 0.0), Point::new(10.0, 0.0)];
        let result = resolve_conflicts(&points, 0.2, 0.15, &mut board, LayerId(1), None);
        assert_eq!(result.len(), 2);
    }

    #[test]
    fn test_simple_push() {
        let mut board = PcbBoard::default();
        // Add existing trace that will be pushed
        let existing = Trace {
            net_id: Some(NetId(1)),
            layer: LayerId(1),
            width: 0.2,
            points: vec![Point::new(5.0, 5.0), Point::new(5.0, -5.0)],
        };
        board.traces.insert(existing);

        // New trace that crosses the existing one
        let new_points = vec![Point::new(-5.0, 0.0), Point::new(5.0, 0.0)];
        let result = resolve_conflicts(&new_points, 0.2, 0.15, &mut board, LayerId(1), None);
        assert_eq!(result.len(), 2);

        // The existing trace should have been pushed
        let (_, pushed) = board.traces.iter().next().unwrap();
        assert!(pushed.points[0].x > 5.0 || pushed.points[0].x < 5.0 ||
                pushed.points[1].x > 5.0 || pushed.points[1].x < 5.0);
    }
}
