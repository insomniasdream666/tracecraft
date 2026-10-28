//! ISO 10303-21 (STEP AP203) mechanical export.
//!
//! Emits a `MANIFOLD_SOLID_BREP` for the board, each copper trace segment, each
//! via and each component body. AP203 consumers (FreeCAD, SolidWorks, Fusion,
//! OnShape, OCCT) accept boundary representations, so the geometry is
//! tessellated rather than emitted as analytic surfaces.

use std::collections::HashMap;

use crate::model::geometry::Point;
use crate::model::library::{Component, ComponentId};
use crate::model::pcb::{LayerKind, PcbBoard};

const COPPER_THICKNESS_MM: f64 = 0.035;
const DEFAULT_COMPONENT_HEIGHT_MM: f64 = 1.6;
const VIA_SIDES: usize = 12;

pub type ComponentMap = HashMap<ComponentId, Component>;

/// Build a complete STEP file for a board.
pub fn generate_step(board: &PcbBoard, components: &ComponentMap) -> Result<String, String> {
    let thickness = board_thickness(board);
    let mut writer = StepWriter::new();
    let mut solids: Vec<usize> = Vec::new();

    // ── Board substrate ────────────────────────────────────────────────────
    let outline = board
        .outline
        .as_ref()
        .and_then(|o| o.shape.first())
        .filter(|pts| pts.len() >= 3)
        .cloned()
        .unwrap_or_else(|| {
            vec![
                Point::new(0.0, 0.0),
                Point::new(100.0, 0.0),
                Point::new(100.0, 80.0),
                Point::new(0.0, 80.0),
            ]
        });

    let outline_xyz: Vec<[f64; 3]> = outline
        .iter()
        .map(|p| [p.x, p.y, 0.0])
        .collect();

    if let Some(solid) = writer.prism(&outline_xyz, 0.0, thickness) {
        solids.push(solid);
    }

    // ── Copper traces ──────────────────────────────────────────────────────
    let copper_z = thickness;
    for (_, trace) in board.traces.iter() {
        if trace.points.len() < 2 || trace.width <= 0.0 {
            continue;
        }
        for pair in trace.points.windows(2) {
            let (a, b) = (pair[0], pair[1]);
            let dx = b.x - a.x;
            let dy = b.y - a.y;
            let len = (dx * dx + dy * dy).sqrt();
            if len < 1e-6 {
                continue;
            }
            let (ux, uy) = (dx / len, dy / len);
            let (vx, vy) = (-uy, ux);
            let half = trace.width / 2.0;
            let corners = [
                Point::new(a.x + vx * half, a.y + vy * half),
                Point::new(b.x + vx * half, b.y + vy * half),
                Point::new(b.x - vx * half, b.y - vy * half),
                Point::new(a.x - vx * half, a.y - vy * half),
            ];
            let polygon: Vec<[f64; 3]> = corners.iter().map(|p| [p.x, p.y, 0.0]).collect();
            if let Some(solid) = writer.prism(&polygon, copper_z, copper_z + COPPER_THICKNESS_MM) {
                solids.push(solid);
            }
        }
    }

    // ── Vias ───────────────────────────────────────────────────────────────
    for (_, via) in board.vias.iter() {
        if via.diameter <= 0.0 {
            continue;
        }
        let radius = via.diameter / 2.0;
        let polygon: Vec<[f64; 3]> = (0..VIA_SIDES)
            .map(|i| {
                let angle = (i as f64) * std::f64::consts::TAU / (VIA_SIDES as f64);
                [
                    via.position.x + radius * angle.cos(),
                    via.position.y + radius * angle.sin(),
                    0.0,
                ]
            })
            .collect();
        if let Some(solid) = writer.prism(&polygon, 0.0, thickness) {
            solids.push(solid);
        }
    }

    // ── Component bodies ───────────────────────────────────────────────────
    for (_, footprint) in board.footprints.iter() {
        let Some(component) = components.get(&footprint.component_id) else {
            continue;
        };
        if component.footprint.outline.len() < 3 {
            continue;
        }
        let angle = footprint.rotation.0.to_radians();
        let (sin, cos) = angle.sin_cos();
        let mut polygon: Vec<[f64; 3]> = Vec::new();
        for point in &component.footprint.outline {
            let x = if footprint.mirrored { -point.x } else { point.x };
            let y = point.y;
            polygon.push([
                footprint.position.x + x * cos - y * sin,
                footprint.position.y + x * sin + y * cos,
                copper_z,
            ]);
        }
        // Drop duplicate closing vertex if the outline repeats its first point.
        if polygon.len() > 3 {
            let first = polygon[0];
            let last = polygon[polygon.len() - 1];
            if (first[0] - last[0]).abs() < 1e-9 && (first[1] - last[1]).abs() < 1e-9 {
                polygon.pop();
            }
        }
        let height = component_height(component);
        if let Some(solid) = writer.prism(&polygon, copper_z, copper_z + height) {
            solids.push(solid);
        }
    }

    if solids.is_empty() {
        return Err("Board has no geometry to export".to_string());
    }

    Ok(writer.finish(&solids))
}

fn board_thickness(board: &PcbBoard) -> f64 {
    let dielectric: f64 = board
        .layers
        .dielectrics
        .iter()
        .map(|d| d.thickness_mm.max(0.0))
        .sum();
    if dielectric > 0.05 {
        let copper_layers = board
            .layers
            .layers
            .iter()
            .filter(|l| {
                matches!(
                    l.kind,
                    LayerKind::Top | LayerKind::Bottom | LayerKind::Inner
                )
            })
            .count() as f64;
        dielectric + copper_layers * COPPER_THICKNESS_MM
    } else {
        1.6
    }
}

fn component_height(component: &Component) -> f64 {
    for key in ["Height", "height", "Height (mm)", "height_mm"] {
        if let Some(value) = component.parameters.get(key) {
            if let Ok(parsed) = value
                .trim()
                .trim_end_matches(|c: char| c.is_ascii_alphabetic())
                .parse::<f64>()
            {
                if parsed > 0.0 {
                    return parsed;
                }
            }
        }
    }
    DEFAULT_COMPONENT_HEIGHT_MM
}

// ── STEP entity writer ─────────────────────────────────────────────────────

struct StepWriter {
    next_id: usize,
    entities: Vec<String>,
}

impl StepWriter {
    fn new() -> Self {
        Self {
            next_id: 1,
            entities: Vec::new(),
        }
    }

    fn add(&mut self, body: String) -> usize {
        let id = self.next_id;
        self.next_id += 1;
        self.entities.push(format!("#{}={};", id, body));
        id
    }

    fn point(&mut self, p: [f64; 3]) -> usize {
        self.add(format!(
            "CARTESIAN_POINT('',({},{},{}))",
            num(p[0]),
            num(p[1]),
            num(p[2])
        ))
    }

    fn direction(&mut self, d: [f64; 3]) -> usize {
        self.add(format!(
            "DIRECTION('',({},{},{}))",
            num(d[0]),
            num(d[1]),
            num(d[2])
        ))
    }

    fn vertex(&mut self, p: [f64; 3]) -> usize {
        let point = self.point(p);
        self.add(format!("VERTEX_POINT('',#{})", point))
    }

    fn edge(&mut self, v1: usize, v2: usize, p1: [f64; 3], p2: [f64; 3]) -> Option<usize> {
        let direction = normalize([p2[0] - p1[0], p2[1] - p1[1], p2[2] - p1[2]])?;
        let dir = self.direction(direction);
        let vector = self.add(format!("VECTOR('',#{},1.)", dir));
        let origin = self.point(p1);
        let line = self.add(format!("LINE('',#{},#{})", origin, vector));
        Some(self.add(format!(
            "EDGE_CURVE('',#{},#{},#{},.T.)",
            v1, v2, line
        )))
    }

    /// Emit an `ADVANCED_FACE` whose loop is built from existing edge curves.
    ///
    /// `oriented` holds `(edge_id, same_direction)` pairs in loop order.
    fn advanced_face(&mut self, oriented: &[(usize, bool)], points: &[[f64; 3]]) -> Option<usize> {
        if points.len() < 3 {
            return None;
        }
        let normal = polygon_normal(points)?;
        let origin = points[0];
        let first_edge = normalize([
            points[1][0] - origin[0],
            points[1][1] - origin[1],
            points[1][2] - origin[2],
        ])?;
        let projection = [
            first_edge[0] - normal[0] * dot(first_edge, normal),
            first_edge[1] - normal[1] * dot(first_edge, normal),
            first_edge[2] - normal[2] * dot(first_edge, normal),
        ];
        let ref_dir = normalize(projection).unwrap_or([1.0, 0.0, 0.0]);

        let origin_id = self.point(origin);
        let axis_id = self.direction(normal);
        let ref_id = self.direction(ref_dir);
        let placement = self.add(format!(
            "AXIS2_PLACEMENT_3D('',#{},#{},#{})",
            origin_id, axis_id, ref_id
        ));
        let plane = self.add(format!("PLANE('',#{})", placement));

        let oriented_ids: Vec<usize> = oriented
            .iter()
            .map(|(edge, forward)| {
                self.add(format!(
                    "ORIENTED_EDGE('',*,*,#{},.{})",
                    edge,
                    if *forward { "T." } else { "F." }
                ))
            })
            .collect();
        let edge_loop = self.add(format!(
            "EDGE_LOOP('',({}))",
            oriented_ids
                .iter()
                .map(|id| format!("#{}", id))
                .collect::<Vec<_>>()
                .join(",")
        ));
        let bound = self.add(format!("FACE_OUTER_BOUND('',#{},.T.)", edge_loop));
        Some(self.add(format!(
            "ADVANCED_FACE('',(#{}),#{},.T.)",
            bound, plane
        )))
    }

    /// Emit a closed solid for an extruded polygon.
    fn prism(&mut self, polygon: &[[f64; 3]], z0: f64, z1: f64) -> Option<usize> {
        let n = polygon.len();
        if n < 3 || (z1 - z0).abs() < 1e-9 {
            return None;
        }
        // Ensure counter-clockwise winding when viewed from +Z.
        let area = signed_area(polygon);
        let mut poly: Vec<[f64; 3]> = polygon.to_vec();
        if area < 0.0 {
            poly.reverse();
        }

        let bottom_pts: Vec<[f64; 3]> = poly.iter().map(|p| [p[0], p[1], z0]).collect();
        let top_pts: Vec<[f64; 3]> = poly.iter().map(|p| [p[0], p[1], z1]).collect();

        let bottom_vertices: Vec<usize> = bottom_pts.iter().map(|p| self.vertex(*p)).collect();
        let top_vertices: Vec<usize> = top_pts.iter().map(|p| self.vertex(*p)).collect();

        let mut bottom_edges = Vec::with_capacity(n);
        let mut top_edges = Vec::with_capacity(n);
        let mut vertical_edges = Vec::with_capacity(n);
        for i in 0..n {
            let j = (i + 1) % n;
            bottom_edges.push(self.edge(
                bottom_vertices[i],
                bottom_vertices[j],
                bottom_pts[i],
                bottom_pts[j],
            )?);
            top_edges.push(self.edge(
                top_vertices[i],
                top_vertices[j],
                top_pts[i],
                top_pts[j],
            )?);
            vertical_edges.push(self.edge(
                bottom_vertices[i],
                top_vertices[i],
                bottom_pts[i],
                top_pts[i],
            )?);
        }

        let mut faces = Vec::with_capacity(n + 2);

        // Bottom: traverse the ring backwards so the normal points down.
        let mut oriented = Vec::with_capacity(n);
        let mut points = Vec::with_capacity(n);
        for k in 0..n {
            let a = (n - k) % n;
            let b = (n + a - 1) % n;
            // bottom_edges[b] runs b -> b+1 = a, so traverse it reversed.
            oriented.push((bottom_edges[b], false));
            points.push(bottom_pts[a]);
        }
        faces.push(self.advanced_face(&oriented, &points)?);

        // Top: forward ring, normal up.
        let oriented: Vec<(usize, bool)> = (0..n).map(|i| (top_edges[i], true)).collect();
        faces.push(self.advanced_face(&oriented, &top_pts)?);

        // Side walls.
        for i in 0..n {
            let j = (i + 1) % n;
            let oriented = vec![
                (bottom_edges[i], true),
                (vertical_edges[j], true),
                (top_edges[i], false),
                (vertical_edges[i], false),
            ];
            let points = vec![bottom_pts[i], bottom_pts[j], top_pts[j], top_pts[i]];
            faces.push(self.advanced_face(&oriented, &points)?);
        }

        let shell = self.add(format!(
            "CLOSED_SHELL('',({}))",
            faces
                .iter()
                .map(|f| format!("#{}", f))
                .collect::<Vec<_>>()
                .join(",")
        ));
        Some(self.add(format!("MANIFOLD_SOLID_BREP('',#{})", shell)))
    }

    fn finish(mut self, solids: &[usize]) -> String {
        let app_context = self.add(
            "APPLICATION_CONTEXT('configuration controlled 3d designs of mechanical parts and assemblies')"
                .to_string(),
        );
        self.add(format!(
            "APPLICATION_PROTOCOL_DEFINITION('international standard','config_control_design',1994,#{})",
            app_context
        ));
        let product_context =
            self.add(format!("PRODUCT_CONTEXT('',#{},'mechanical')", app_context));
        let product = self.add(format!(
            "PRODUCT('TraceCraft PCB','TraceCraft PCB','',(#{}))",
            product_context
        ));
        let formation = self.add(format!(
            "PRODUCT_DEFINITION_FORMATION_WITH_SPECIFIED_SOURCE('','',#{},.NOT_KNOWN.)",
            product
        ));
        let definition_context = self.add(format!(
            "PRODUCT_DEFINITION_CONTEXT('part definition',#{},'design')",
            app_context
        ));
        let definition = self.add(format!(
            "PRODUCT_DEFINITION('design','',#{},#{})",
            formation, definition_context
        ));
        let shape = self.add(format!("PRODUCT_DEFINITION_SHAPE('','',#{})", definition));

        let length_unit = self.add(
            "(LENGTH_UNIT() NAMED_UNIT(*) SI_UNIT(.MILLI.,.METRE.))".to_string(),
        );
        let angle_unit = self.add(
            "(NAMED_UNIT(*) PLANE_ANGLE_UNIT() SI_UNIT($,.RADIAN.))".to_string(),
        );
        let solid_angle_unit = self.add(
            "(NAMED_UNIT(*) SI_UNIT($,.STERADIAN.) SOLID_ANGLE_UNIT())".to_string(),
        );
        let uncertainty = self.add(format!(
            "UNCERTAINTY_MEASURE_WITH_UNIT(LENGTH_MEASURE(1.E-6),#{},'distance_accuracy_value','confusion accuracy')",
            length_unit
        ));
        let context = self.add(format!(
            "(GEOMETRIC_REPRESENTATION_CONTEXT(3) GLOBAL_UNCERTAINTY_ASSIGNED_CONTEXT((#{})) GLOBAL_UNIT_ASSIGNED_CONTEXT((#{},#{},#{})) REPRESENTATION_CONTEXT('Context','3D'))",
            uncertainty, length_unit, angle_unit, solid_angle_unit
        ));

        let origin = self.point([0.0, 0.0, 0.0]);
        let z_axis = self.direction([0.0, 0.0, 1.0]);
        let x_axis = self.direction([1.0, 0.0, 0.0]);
        let placement = self.add(format!(
            "AXIS2_PLACEMENT_3D('',#{},#{},#{})",
            origin, z_axis, x_axis
        ));

        let mut items: Vec<String> = solids.iter().map(|id| format!("#{}", id)).collect();
        items.push(format!("#{}", placement));
        let representation = self.add(format!(
            "ADVANCED_BREP_SHAPE_REPRESENTATION('TraceCraft PCB',({}),#{})",
            items.join(","),
            context
        ));
        self.add(format!(
            "SHAPE_DEFINITION_REPRESENTATION(#{},#{})",
            shape, representation
        ));

        let mut out = String::new();
        out.push_str("ISO-10303-21;\n");
        out.push_str("HEADER;\n");
        out.push_str("FILE_DESCRIPTION(('TraceCraft PCB mechanical model'),'2;1');\n");
        out.push_str(&format!(
            "FILE_NAME('tracecraft.step','{}',('TraceCraft'),('TraceCraft'),'TraceCraft','TraceCraft','');\n",
            chrono::Utc::now().format("%Y-%m-%dT%H:%M:%S")
        ));
        out.push_str("FILE_SCHEMA(('CONFIG_CONTROL_DESIGN'));\n");
        out.push_str("ENDSEC;\n\nDATA;\n");
        for entity in &self.entities {
            out.push_str(entity);
            out.push('\n');
        }
        out.push_str("ENDSEC;\nEND-ISO-10303-21;\n");
        out
    }
}

fn signed_area(points: &[[f64; 3]]) -> f64 {
    let mut area = 0.0;
    for i in 0..points.len() {
        let j = (i + 1) % points.len();
        area += points[i][0] * points[j][1] - points[j][0] * points[i][1];
    }
    area / 2.0
}

fn polygon_normal(points: &[[f64; 3]]) -> Option<[f64; 3]> {
    // Newell's method: robust for non-planar or near-degenerate loops.
    let mut normal = [0.0f64; 3];
    for i in 0..points.len() {
        let j = (i + 1) % points.len();
        let a = points[i];
        let b = points[j];
        normal[0] += (a[1] - b[1]) * (a[2] + b[2]);
        normal[1] += (a[2] - b[2]) * (a[0] + b[0]);
        normal[2] += (a[0] - b[0]) * (a[1] + b[1]);
    }
    normalize(normal)
}

fn normalize(v: [f64; 3]) -> Option<[f64; 3]> {
    let len = (v[0] * v[0] + v[1] * v[1] + v[2] * v[2]).sqrt();
    if len < 1e-12 {
        None
    } else {
        Some([v[0] / len, v[1] / len, v[2] / len])
    }
}

fn dot(a: [f64; 3], b: [f64; 3]) -> f64 {
    a[0] * b[0] + a[1] * b[1] + a[2] * b[2]
}

fn num(v: f64) -> String {
    let v = if v.abs() < 1e-9 { 0.0 } else { v };
    let text = format!("{:.6}", v);
    let trimmed = text.trim_end_matches('0').trim_end_matches('.');
    if trimmed.is_empty() || trimmed == "-0" {
        "0".to_string()
    } else {
        trimmed.to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::pcb::{BoardOutline, PcbBoard, Trace, Via};
    use crate::model::id::LayerId;

    fn sample_board() -> PcbBoard {
        let mut board = PcbBoard::default();
        board.outline = Some(BoardOutline {
            shape: vec![vec![
                Point::new(0.0, 0.0),
                Point::new(50.0, 0.0),
                Point::new(50.0, 40.0),
                Point::new(0.0, 40.0),
            ]],
            cutouts: vec![],
        });
        board.traces.insert(Trace {
            net_id: None,
            layer: LayerId(1),
            width: 0.25,
            points: vec![Point::new(5.0, 5.0), Point::new(20.0, 5.0)],
            attenuation_db_per_m: 0.0,
        });
        board.vias.insert(Via {
            position: Point::new(25.0, 25.0),
            start_layer: LayerId(1),
            end_layer: LayerId(2),
            diameter: 0.6,
            drill: 0.3,
            net_id: None,
        });
        board
    }

    #[test]
    fn emits_ap203_structure() {
        let step = generate_step(&sample_board(), &ComponentMap::new()).unwrap();
        assert!(step.starts_with("ISO-10303-21;"));
        assert!(step.contains("FILE_SCHEMA(('CONFIG_CONTROL_DESIGN'));"));
        assert!(step.contains("ADVANCED_BREP_SHAPE_REPRESENTATION"));
        assert!(step.contains("MANIFOLD_SOLID_BREP"));
        assert!(step.trim_end().ends_with("END-ISO-10303-21;"));
        assert!(!step.contains("BLOCK("), "STEP has no BLOCK entity");
    }

    #[test]
    fn every_entity_reference_resolves() {
        let step = generate_step(&sample_board(), &ComponentMap::new()).unwrap();
        let mut defined = std::collections::HashSet::new();
        for line in step.lines().filter(|l| l.starts_with('#')) {
            if let Some((id, _)) = line.split_once('=') {
                defined.insert(id.trim().to_string());
            }
        }
        assert!(!defined.is_empty());
        for line in step.lines().filter(|l| l.starts_with('#')) {
            let mut rest = line;
            while let Some(pos) = rest.find('#') {
                rest = &rest[pos + 1..];
                let end = rest
                    .find(|c: char| !c.is_ascii_digit())
                    .unwrap_or(rest.len());
                if end == 0 {
                    break;
                }
                let id = format!("#{}", &rest[..end]);
                assert!(defined.contains(&id), "unresolved reference {} in {}", id, line);
                rest = &rest[end..];
            }
        }
    }

    #[test]
    fn solid_count_matches_geometry() {
        let step = generate_step(&sample_board(), &ComponentMap::new()).unwrap();
        // board + trace segment + via
        assert_eq!(step.matches("MANIFOLD_SOLID_BREP(").count(), 3);
        assert_eq!(step.matches("CLOSED_SHELL(").count(), 3);
    }

    #[test]
    fn empty_board_still_exports_outline() {
        let mut board = PcbBoard::default();
        board.outline = Some(BoardOutline {
            shape: vec![vec![
                Point::new(0.0, 0.0),
                Point::new(10.0, 0.0),
                Point::new(10.0, 10.0),
            ]],
            cutouts: vec![],
        });
        let step = generate_step(&board, &ComponentMap::new()).unwrap();
        assert_eq!(step.matches("MANIFOLD_SOLID_BREP(").count(), 1);
    }
}
