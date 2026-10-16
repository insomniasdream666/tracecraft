use crate::model::geometry::Point;
use crate::model::pcb::{PcbBoard, Via};
use crate::model::id::{ViaId, NetId, LayerId};

/// Generate a ground via stitch along a line segment.
/// Places vias at regular intervals along the segment.
pub fn stitch_via_line(
    board: &mut PcbBoard,
    start: Point,
    end: Point,
    net_id: NetId,
    spacing: f64,
    via_diameter: f64,
    via_drill: f64,
    start_layer: LayerId,
    end_layer: LayerId,
) -> Vec<ViaId> {
    if spacing <= 0.0 {
        return vec![];
    }
    let dx = end.x - start.x;
    let dy = end.y - start.y;
    let len = (dx * dx + dy * dy).sqrt();
    if len < spacing {
        return vec![];
    }

    let count = (len / spacing).ceil() as usize;
    let ux = dx / len;
    let uy = dy / len;
    let mut ids = Vec::new();

    for i in 0..=count {
        let t = (i as f64) / (count as f64);
        let pos = Point::new(start.x + ux * t * len, start.y + uy * t * len);
        let via = Via {
            position: pos,
            start_layer,
            end_layer,
            diameter: via_diameter,
            drill: via_drill,
            net_id: Some(net_id),
        };
        ids.push(ViaId(board.vias.insert(via)));
    }

    ids
}

/// Generate a ground via shield around a set of points (trace path).
/// Places vias offset to both sides of the trace.
pub fn stitch_via_shield(
    board: &mut PcbBoard,
    trace_points: &[Point],
    net_id: NetId,
    spacing: f64,
    offset: f64,
    via_diameter: f64,
    via_drill: f64,
    start_layer: LayerId,
    end_layer: LayerId,
) -> Vec<ViaId> {
    if trace_points.len() < 2 || spacing <= 0.0 {
        return vec![];
    }

    let mut ids = Vec::new();
    for i in 0..trace_points.len() - 1 {
        let a = trace_points[i];
        let b = trace_points[i + 1];
        let dx = b.x - a.x;
        let dy = b.y - a.y;
        let len = (dx * dx + dy * dy).sqrt();
        if len < 1e-6 { continue; }

        let nx = -dy / len * offset;
        let ny = dx / len * offset;

        // Via on each side at the segment start
        let left = Point::new(a.x + nx, a.y + ny);
        let right = Point::new(a.x - nx, a.y - ny);

        for pos in [left, right] {
            let via = Via {
                position: pos,
                start_layer,
                end_layer,
                diameter: via_diameter,
                drill: via_drill,
                net_id: Some(net_id),
            };
            ids.push(ViaId(board.vias.insert(via)));
        }

        // Also add along the segment at spacing intervals
        let seg_steps = (len / spacing).ceil() as usize;
        for j in 1..seg_steps {
            let t = j as f64 / seg_steps.max(1) as f64;
            let px = a.x + dx * t;
            let py = a.y + dy * t;
            for sign in [-1.0, 1.0] {
                let via = Via {
                    position: Point::new(px + nx * sign, py + ny * sign),
                    start_layer,
                    end_layer,
                    diameter: via_diameter,
                    drill: via_drill,
                    net_id: Some(net_id),
                };
                ids.push(ViaId(board.vias.insert(via)));
            }
        }
    }

    // Add vias at the final trace point if it wasn't already covered
    let last = trace_points.last().unwrap();
    let second_last = trace_points[trace_points.len() - 2];
    let dx = last.x - second_last.x;
    let dy = last.y - second_last.y;
    let len = (dx * dx + dy * dy).sqrt();
    if len > 1e-6 {
        let nx = -dy / len * offset;
        let ny = dx / len * offset;
        for sign in [-1.0, 1.0] {
            let via = Via {
                position: Point::new(last.x + nx * sign, last.y + ny * sign),
                start_layer,
                end_layer,
                diameter: via_diameter,
                drill: via_drill,
                net_id: Some(net_id),
            };
            ids.push(ViaId(board.vias.insert(via)));
        }
    }

    ids
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::pcb::PcbBoard;

    #[test]
    fn test_stitch_via_line() {
        let mut board = PcbBoard::default();
        let ids = stitch_via_line(
            &mut board,
            Point::new(0.0, 0.0),
            Point::new(10.0, 0.0),
            NetId(0),
            2.0,
            0.6,
            0.3,
            LayerId(1),
            LayerId(2),
        );
        assert!(ids.len() >= 5); // 10mm / 2mm spacing = 5 + 1 = 6
        assert_eq!(board.vias.len(), ids.len());
    }

    #[test]
    fn test_stitch_via_shield() {
        let mut board = PcbBoard::default();
        let points = vec![Point::new(0.0, 0.0), Point::new(10.0, 0.0)];
        let ids = stitch_via_shield(
            &mut board,
            &points,
            NetId(0),
            2.0,
            1.0,
            0.6,
            0.3,
            LayerId(1),
            LayerId(2),
        );
        assert!(!ids.is_empty());
    }
}
