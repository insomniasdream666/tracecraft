use crate::model::pcb::*;
use crate::model::geometry::*;
use crate::model::id::*;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DRCViolation {
    pub severity: Severity,
    pub rule: DRCRule,
    pub description: String,
    pub location: Point,
    pub layer: LayerId,
    pub element_a: ElementRef,
    pub element_b: ElementRef,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Severity {
    Error,
    Warning,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum DRCRule {
    TraceWidth,
    TraceClearance,
    PadClearance,
    AnnularRing,
    HoleSize,
    SilkscreenOverlap,
    UnroutedNet,
    SolderMaskSliver,
    CopperBalancing,
    AcidTrap,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ElementRef {
    pub kind: ElementKind,
    pub id: usize,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ElementKind {
    Trace,
    Via,
    Footprint,
    Zone,
    Pad,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DRCSettings {
    pub enabled_checks: Vec<DRCRule>,
}

impl Default for DRCSettings {
    fn default() -> Self {
        Self {
            enabled_checks: vec![
                DRCRule::TraceWidth,
                DRCRule::TraceClearance,
                DRCRule::PadClearance,
                DRCRule::AnnularRing,
                DRCRule::HoleSize,
                DRCRule::SilkscreenOverlap,
            ],
        }
    }
}

// ─── Geometry Helpers ──────────────────────────────────────────────────────

fn segment_distance_squared(p1: &Point, p2: &Point, q: &Point) -> f64 {
    let dx = p2.x - p1.x;
    let dy = p2.y - p1.y;
    let len2 = dx * dx + dy * dy;
    if len2 == 0.0 {
        return p1.distance(q).powi(2);
    }
    let t = ((q.x - p1.x) * dx + (q.y - p1.y) * dy) / len2;
    let t = t.clamp(0.0, 1.0);
    let proj = Point::new(p1.x + t * dx, p1.y + t * dy);
    proj.distance(q).powi(2)
}

fn segments_min_distance(a1: &Point, a2: &Point, b1: &Point, b2: &Point) -> f64 {
    let d1 = segment_distance_squared(a1, a2, b1).sqrt();
    let d2 = segment_distance_squared(a1, a2, b2).sqrt();
    let d3 = segment_distance_squared(b1, b2, a1).sqrt();
    let d4 = segment_distance_squared(b1, b2, a2).sqrt();
    d1.min(d2).min(d3).min(d4)
}

fn pad_position(fp: &FootprintInstance) -> Point {
    fp.position
}

// Estimate pad bounding radius from footprint size
fn pad_radius(_fp: &FootprintInstance) -> f64 {
    5.0
}

// ─── DRC Checks ────────────────────────────────────────────────────────────

pub fn run_drc_checks(board: &PcbBoard) -> Vec<DRCViolation> {
    let mut violations = Vec::new();
    let rules = &board.design_rules;

    // 1. Trace width check
    for (id, trace) in board.traces.iter() {
        if trace.width < rules.min_trace_width {
            let center = trace_center(&trace);
            violations.push(DRCViolation {
                severity: Severity::Error,
                rule: DRCRule::TraceWidth,
                description: format!(
                    "Trace width {:.3}mm is below minimum {:.3}mm",
                    trace.width, rules.min_trace_width
                ),
                location: center,
                layer: trace.layer,
                element_a: ElementRef { kind: ElementKind::Trace, id },
                element_b: ElementRef { kind: ElementKind::Trace, id },
            });
        }
    }

    // 2. Trace-to-trace clearance
    let traces: Vec<(usize, &Trace)> = board.traces.iter().collect();
    for i in 0..traces.len() {
        for j in (i + 1)..traces.len() {
            let (id_a, a) = traces[i];
            let (id_b, b) = traces[j];

            // Only check same-layer traces
            if a.layer != b.layer {
                continue;
            }

            for seg_a in a.points.windows(2) {
                let a1 = &seg_a[0];
                let a2 = &seg_a[1];
                let half_w_a = a.width / 2.0;

                for seg_b in b.points.windows(2) {
                    let b1 = &seg_b[0];
                    let b2 = &seg_b[1];
                    let half_w_b = b.width / 2.0;

                    let edge_dist = segments_min_distance(a1, a2, b1, b2);
                    let clearance = edge_dist - half_w_a - half_w_b;

                    if clearance < rules.min_clearance {
                        let mid = Point::new((a1.x + a2.x) / 2.0, (a1.y + a2.y) / 2.0);
                        violations.push(DRCViolation {
                            severity: Severity::Error,
                            rule: DRCRule::TraceClearance,
                            description: format!(
                                "Trace clearance {:.3}mm below minimum {:.3}mm",
                                clearance, rules.min_clearance
                            ),
                            location: mid,
                            layer: a.layer,
                            element_a: ElementRef { kind: ElementKind::Trace, id: id_a },
                            element_b: ElementRef { kind: ElementKind::Trace, id: id_b },
                        });
                    }
                }
            }
        }
    }

    // 3. Trace-to-footprint pad clearance
    for (trace_id, trace) in board.traces.iter() {
        for (fp_id, fp) in board.footprints.iter() {
            let pad_center = pad_position(fp);
            let pad_r = pad_radius(fp);

            for seg in trace.points.windows(2) {
                let d = segment_distance_squared(&seg[0], &seg[1], &pad_center).sqrt();
                let clearance = d - trace.width / 2.0 - pad_r;

                if clearance < rules.min_clearance {
                    violations.push(DRCViolation {
                        severity: Severity::Error,
                        rule: DRCRule::PadClearance,
                        description: format!(
                            "Pad-trace clearance {:.3}mm below minimum {:.3}mm",
                            clearance, rules.min_clearance
                        ),
                        location: pad_center,
                        layer: trace.layer,
                        element_a: ElementRef { kind: ElementKind::Trace, id: trace_id },
                        element_b: ElementRef { kind: ElementKind::Footprint, id: fp_id },
                    });
                }
            }
        }
    }

    // 4. Via annular ring check
    for (id, via) in board.vias.iter() {
        let ring = (via.diameter - via.drill) / 2.0;
        if ring < rules.min_annular_ring {
            violations.push(DRCViolation {
                severity: Severity::Error,
                rule: DRCRule::AnnularRing,
                description: format!(
                    "Via annular ring {:.3}mm below minimum {:.3}mm",
                    ring, rules.min_annular_ring
                ),
                location: via.position,
                layer: via.start_layer,
                element_a: ElementRef { kind: ElementKind::Via, id },
                element_b: ElementRef { kind: ElementKind::Via, id },
            });
        }
    }

    // 5. Via hole size check

    // 6. DFM: Check for acute angles (acid traps) in traces
    for (id, trace) in board.traces.iter() {
        for window in trace.points.windows(3) {
            let a = &window[0];
            let b = &window[1];
            let c = &window[2];
            let angle = interior_angle(a, b, c);
            if angle > 0.0 && angle < 45.0 {
                violations.push(DRCViolation {
                    severity: Severity::Warning,
                    rule: DRCRule::AcidTrap,
                    description: format!("Acute angle {:.1}° in trace — potential acid trap", angle),
                    location: *b,
                    layer: trace.layer,
                    element_a: ElementRef { kind: ElementKind::Trace, id },
                    element_b: ElementRef { kind: ElementKind::Trace, id },
                });
            }
        }
    }

    // 7. DFM: Check solder mask slivers — close parallel traces on outer layers
    let outer_layers = [LayerId(1), LayerId(2), LayerId(10), LayerId(11)];
    for layer in &outer_layers {
        let layer_traces: Vec<(usize, &Trace)> = board.traces.iter()
            .filter(|(_, t)| t.layer == *layer)
            .collect();
        for i in 0..layer_traces.len() {
            for j in (i + 1)..layer_traces.len() {
                let (id_a, a) = layer_traces[i];
                let (id_b, b) = layer_traces[j];
                for seg_a in a.points.windows(2) {
                    for seg_b in b.points.windows(2) {
                        let d = segments_min_distance(&seg_a[0], &seg_a[1], &seg_b[0], &seg_b[1]);
                        let gap = d - a.width / 2.0 - b.width / 2.0;
                        if gap > 0.0 && gap < 0.1 {
                            violations.push(DRCViolation {
                                severity: Severity::Warning,
                                rule: DRCRule::SolderMaskSliver,
                                description: format!("Solder mask sliver: {:.3}mm gap", gap),
                                location: Point::new(
                                    (seg_a[0].x + seg_a[1].x + seg_b[0].x + seg_b[1].x) / 4.0,
                                    (seg_a[0].y + seg_a[1].y + seg_b[0].y + seg_b[1].y) / 4.0,
                                ),
                                layer: *layer,
                                element_a: ElementRef { kind: ElementKind::Trace, id: id_a },
                                element_b: ElementRef { kind: ElementKind::Trace, id: id_b },
                            });
                        }
                    }
                }
            }
        }
    }
    for (id, via) in board.vias.iter() {
        if via.drill < rules.min_hole_size {
            violations.push(DRCViolation {
                severity: Severity::Error,
                rule: DRCRule::HoleSize,
                description: format!(
                    "Via hole size {:.3}mm below minimum {:.3}mm",
                    via.drill, rules.min_hole_size
                ),
                location: via.position,
                layer: via.start_layer,
                element_a: ElementRef { kind: ElementKind::Via, id },
                element_b: ElementRef { kind: ElementKind::Via, id },
            });
        }
    }

    violations
}

fn interior_angle(a: &Point, b: &Point, c: &Point) -> f64 {
    let ba_x = a.x - b.x;
    let ba_y = a.y - b.y;
    let bc_x = c.x - b.x;
    let bc_y = c.y - b.y;
    let dot = ba_x * bc_x + ba_y * bc_y;
    let len_a = (ba_x * ba_x + ba_y * ba_y).sqrt();
    let len_c = (bc_x * bc_x + bc_y * bc_y).sqrt();
    if len_a < 1e-9 || len_c < 1e-9 { return 180.0; }
    let cos_angle = (dot / (len_a * len_c)).clamp(-1.0, 1.0);
    cos_angle.acos().to_degrees()
}

fn trace_center(trace: &Trace) -> Point {
    if trace.points.is_empty() {
        return Point::new(0.0, 0.0);
    }
    let sum: Point = trace
        .points
        .iter()
        .fold(Point::new(0.0, 0.0), |acc, p| Point::new(acc.x + p.x, acc.y + p.y));
    Point::new(
        sum.x / trace.points.len() as f64,
        sum.y / trace.points.len() as f64,
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::id::LayerId;

    fn make_board() -> PcbBoard {
        PcbBoard::default()
    }

    #[test]
    fn test_no_violations_on_empty_board() {
        let board = make_board();
        let v = run_drc_checks(&board);
        assert!(v.is_empty());
    }

    #[test]
    fn test_trace_width_violation() {
        let mut board = make_board();
        board.traces.insert(Trace {
            net_id: None,
            layer: LayerId(1),
            width: 0.01,
            points: vec![Point::new(0.0, 0.0), Point::new(10.0, 0.0)],
            attenuation_db_per_m: 0.0,
        });
        let v = run_drc_checks(&board);
        assert!(v.iter().any(|v| v.rule == DRCRule::TraceWidth));
    }

    #[test]
    fn test_trace_clearance_violation() {
        let mut board = make_board();
        board.traces.insert(Trace {
            net_id: None, layer: LayerId(1), width: 0.25,
            points: vec![Point::new(0.0, 0.0), Point::new(10.0, 0.0)],
            attenuation_db_per_m: 0.0,
        });
        board.traces.insert(Trace {
            net_id: None, layer: LayerId(1), width: 0.25,
            points: vec![Point::new(0.0, 0.15), Point::new(10.0, 0.15)],
            attenuation_db_per_m: 0.0,
        });
        let v = run_drc_checks(&board);
        assert!(v.iter().any(|v| v.rule == DRCRule::TraceClearance));
    }

    #[test]
    fn test_annular_ring_violation() {
        let mut board = make_board();
        board.vias.insert(Via {
            position: Point::new(50.0, 50.0),
            start_layer: LayerId(1), end_layer: LayerId(2),
            diameter: 0.4, drill: 0.35,
            net_id: None,
        });
        let v = run_drc_checks(&board);
        assert!(v.iter().any(|v| v.rule == DRCRule::AnnularRing));
    }

    #[test]
    fn test_hole_size_violation() {
        let mut board = make_board();
        board.vias.insert(Via {
            position: Point::new(50.0, 50.0),
            start_layer: LayerId(1), end_layer: LayerId(2),
            diameter: 0.6, drill: 0.15,
            net_id: None,
        });
        let v = run_drc_checks(&board);
        assert!(v.iter().any(|v| v.rule == DRCRule::HoleSize));
    }

    #[test]
    fn test_segment_distance() {
        let p1 = Point::new(0.0, 0.0);
        let p2 = Point::new(10.0, 0.0);
        let q = Point::new(5.0, 3.0);
        let d = segment_distance_squared(&p1, &p2, &q).sqrt();
        assert!((d - 3.0).abs() < 1e-6);
    }

    #[test]
    fn test_segments_min_distance_parallel() {
        let d = segments_min_distance(
            &Point::new(0.0, 0.0), &Point::new(10.0, 0.0),
            &Point::new(0.0, 1.0), &Point::new(10.0, 1.0),
        );
        assert!((d - 1.0).abs() < 1e-6);
    }
}
