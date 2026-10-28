use crate::model::geometry::{Point, Rotation};
use crate::model::id::{ComponentId, LayerId};
use crate::model::pcb::*;
use super::kicad_sch::{self, Sexpr};

pub fn import_kicad_pcb(path: &str) -> Result<PcbBoard, String> {
    let text = std::fs::read_to_string(path).map_err(|e| e.to_string())?;
    let parsed = kicad_sch::parse_sexpr(&text)?;
    let root = Sexpr::List(parsed);

    let mut board = PcbBoard::default();

    let mut shape: Vec<Point> = Vec::new();
    for edge in kicad_sch::find_all(&root, "gr_line") {
        let start = kicad_sch::find_attr(edge, "start");
        let end = kicad_sch::find_attr(edge, "end");
        let layer = kicad_sch::get_str(edge, 1).unwrap_or("");
        if layer == "Edge.Cuts" {
            if let Some(s) = start { shape.push(Point { x: s.0, y: s.1 }); }
            if let Some(e) = end { shape.push(Point { x: e.0, y: e.1 }); }
        }
    }
    for poly in kicad_sch::find_all(&root, "gr_poly") {
        let pts: Vec<Point> = kicad_sch::find_all(poly, "xy").iter()
            .filter_map(|xy| {
                let x = kicad_sch::get_float(xy, 0)?;
                let y = kicad_sch::get_float(xy, 1)?;
                Some(Point { x, y })
            }).collect();
        let layer = kicad_sch::get_str(poly, 1).unwrap_or("");
        if layer == "Edge.Cuts" && pts.len() >= 2 {
            shape.extend(pts);
        }
    }
    board.outline = Some(BoardOutline {
        shape: vec![shape],
        cutouts: vec![],
    });

    for seg in kicad_sch::find_all(&root, "segment") {
        let start = kicad_sch::find_attr(seg, "start");
        let end = kicad_sch::find_attr(seg, "end");
        let width = kicad_sch::find_attr_float(seg, "width").unwrap_or(0.254);
        let layer_name = kicad_sch::get_str(seg, 1).unwrap_or("F.Cu");
        let layer = match layer_name {
            "B.Cu" => LayerId(2),
            _ => LayerId(1),
        };
        if let (Some((x1, y1)), Some((x2, y2))) = (start, end) {
            board.traces.insert(Trace {
                net_id: None,
                layer,
                width,
                points: vec![Point { x: x1, y: y1 }, Point { x: x2, y: y2 }],
                attenuation_db_per_m: 0.0,
            });
        }
    }

    for fp in kicad_sch::find_all(&root, "footprint") {
        let at = kicad_sch::find_attr(fp, "at");
        let pos = at.unwrap_or((0.0, 0.0));
        let refdes = kicad_sch::find_first_child(fp, "Reference")
            .unwrap_or_else(|| "U?".to_string());
        board.footprints.insert(FootprintInstance {
            component_id: ComponentId(uuid::Uuid::new_v4()),
            position: Point { x: pos.0, y: pos.1 },
            rotation: Rotation(0.0),
            mirrored: false,
            refdes,
            value: String::new(),
            locked: false,
        });
    }

    Ok(board)
}

fn parse_pad_shape(s: &str) -> crate::model::library::PadShape {
    match s {
        "rect" => crate::model::library::PadShape::Rect,
        "oval" => crate::model::library::PadShape::Oval,
        "roundrect" => crate::model::library::PadShape::RoundRect,
        "trapezoid" | "custom" => crate::model::library::PadShape::Custom,
        _ => crate::model::library::PadShape::Round,
    }
}

fn layer_name_to_id(name: &str) -> LayerId {
    match name {
        "F.Cu" | "Top" => LayerId(1),
        "B.Cu" | "Bottom" => LayerId(2),
        "F.SilkS" => LayerId(10),
        "B.SilkS" => LayerId(11),
        "F.Mask" | "F.SolderMask" => LayerId(20),
        "B.Mask" | "B.SolderMask" => LayerId(21),
        "F.Paste" => LayerId(30),
        "B.Paste" => LayerId(31),
        "F.Fab" => LayerId(40),
        "B.Fab" => LayerId(41),
        "F.CrtYd" => LayerId(50),
        "B.CrtYd" => LayerId(51),
        "Edge.Cuts" => LayerId(60),
        "*.Cu" => LayerId(1),
        _ => LayerId(1),
    }
}

fn parse_pad_expr(expr: &Sexpr) -> Option<crate::model::library::PadDef> {
    let number = kicad_sch::get_str(expr, 1)?.to_string();
    let pad_type = kicad_sch::get_str(expr, 2).unwrap_or("smd");
    let shape_str = kicad_sch::get_str(expr, 3).unwrap_or("rect");
    let shape = parse_pad_shape(shape_str);
    let pos = kicad_sch::find_attr(expr, "at").unwrap_or((0.0, 0.0));
    let size_xy = kicad_sch::find_attr(expr, "size").unwrap_or((1.0, 1.0));
    let drill = if pad_type == "thru_hole" {
        kicad_sch::find_attr_float(expr, "drill")
    } else {
        None
    };
    let layer_str = kicad_sch::get_str(expr, 4).unwrap_or("F.Cu");
    let layer = layer_name_to_id(layer_str);

    Some(crate::model::library::PadDef {
        number,
        shape,
        position: Point { x: pos.0, y: pos.1 },
        size: crate::model::geometry::Size { width: size_xy.0, height: size_xy.1 },
        layer,
        drill,
    })
}

fn parse_fp_line_ends(expr: &Sexpr) -> (Option<Point>, Option<Point>) {
    let start = kicad_sch::find_attr(expr, "start");
    let end = kicad_sch::find_attr(expr, "end");
    let s = start.map(|(x, y)| Point { x, y });
    let e = end.map(|(x, y)| Point { x, y });
    (s, e)
}

pub fn parse_footprint_def(expr: &Sexpr) -> crate::model::library::FootprintDef {
    let mut pads = Vec::new();
    let mut outline = Vec::new();
    let mut courtyard = Vec::new();

    for pad_expr in kicad_sch::find_all(expr, "pad") {
        if let Some(pad) = parse_pad_expr(pad_expr) {
            pads.push(pad);
        }
    }

    for line_expr in kicad_sch::find_all(expr, "fp_line") {
        let layer = kicad_sch::get_str(line_expr, 1).unwrap_or("");
        let (start, end) = parse_fp_line_ends(line_expr);
        match layer {
            "Edge.Cuts" => {
                if let Some(s) = start { outline.push(s); }
                if let Some(e) = end { outline.push(e); }
            }
            "F.CrtYd" | "B.CrtYd" => {
                if let Some(s) = start { courtyard.push(s); }
                if let Some(e) = end { courtyard.push(e); }
            }
            _ => {}
        }
    }

    crate::model::library::FootprintDef { pads, outline, courtyard }
}

pub fn parse_kicad_mod(path: &str) -> Result<(String, crate::model::library::FootprintDef, Option<crate::model::library::Model3D>), String> {
    let text = std::fs::read_to_string(path).map_err(|e| format!("Failed to read {}: {}", path, e))?;
    let parsed = kicad_sch::parse_sexpr(&text)?;
    let root = Sexpr::List(parsed);

    let name = kicad_sch::get_str(&root, 1).unwrap_or("Unknown").to_string();
    let fp_def = parse_footprint_def(&root);

    let model_3d = kicad_sch::find_all(&root, "model").first().and_then(|m| {
        let model_path = kicad_sch::get_str(m, 1)?;
        Some(crate::model::library::Model3D {
            path: model_path.to_string(),
            offset: Point::new(0.0, 0.0),
            rotation: Rotation(0.0),
        })
    });

    Ok((name, fp_def, model_3d))
}

pub fn import_kicad_pretty_directory(dir: &str) -> Result<Vec<(String, crate::model::library::FootprintDef, Option<crate::model::library::Model3D>)>, String> {
    let mut results = Vec::new();
    let entries = std::fs::read_dir(dir).map_err(|e| format!("Failed to read directory {}: {}", dir, e))?;
    for entry in entries {
        let entry = entry.map_err(|e| e.to_string())?;
        let path = entry.path();
        if path.extension().and_then(|s| s.to_str()) == Some("kicad_mod") {
            let path_str = path.to_str().ok_or_else(|| "Invalid UTF-8 in path".to_string())?;
            match parse_kicad_mod(path_str) {
                Ok((name, fp_def, model)) => results.push((name, fp_def, model)),
                Err(e) => eprintln!("Warning: failed to parse {}: {}", path.display(), e),
            }
        }
    }
    Ok(results)
}

pub fn export_kicad_pcb(board: &PcbBoard, path: &str) -> Result<(), String> {
    let mut out = String::new();
    out.push_str("(kicad_pcb (version 20230121) (generator \"TraceCraft\")\n");
    out.push_str("  (layers (0 \"F.Cu\" signal) (31 \"B.Cu\" signal) (39 \"Edge.Cuts\" user))\n");
    out.push_str("  (setup (pad_to_mask_clearance 0.1))\n");

    if let Some(outline) = &board.outline {
        for contour in &outline.shape {
            if contour.len() >= 2 {
                out.push_str("  (gr_poly (pts");
                for pt in contour {
                    out.push_str(&format!(" (xy {} {})", pt.x, pt.y));
                }
                out.push_str(") (layer Edge.Cuts) (width 0.1))\n");
            }
        }
    }

    for (_, trace) in board.traces.iter() {
        let layer_name = match trace.layer.0 {
            2 => "B.Cu",
            _ => "F.Cu",
        };
        for segment in trace.points.windows(2) {
            out.push_str(&format!(
                "  (segment (start {} {}) (end {} {}) (width {}) (layer {}))\n",
                segment[0].x, segment[0].y,
                segment[1].x, segment[1].y,
                trace.width, layer_name
            ));
        }
    }

    for (_, fp) in board.footprints.iter() {
        out.push_str(&format!("  (footprint {} (layer F.Cu) (at {} {} 0)\n",
            kicad_sch::escape_sexpr(&fp.refdes), fp.position.x, fp.position.y));
        out.push_str(&format!(
            "    (property \"Reference\" {}) (property \"Value\" {})\n",
            kicad_sch::escape_sexpr(&fp.refdes),
            kicad_sch::escape_sexpr(&fp.value)
        ));
        out.push_str("  )\n");
    }

    out.push_str(")\n");
    std::fs::write(path, out).map_err(|e| e.to_string())?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::geometry::{Point, Rotation};
    use crate::model::id::{ComponentId, LayerId};
    use crate::model::pcb::FootprintInstance;

    #[test]
    fn exports_every_segment_and_footprint_value() {
        let mut board = PcbBoard::default();
        board.traces.insert(crate::model::pcb::Trace {
            net_id: None,
            layer: LayerId(1),
            width: 0.25,
            points: vec![Point::new(0.0, 0.0), Point::new(5.0, 0.0), Point::new(5.0, 5.0)],
            attenuation_db_per_m: 0.0,
        });
        board.footprints.insert(FootprintInstance {
            component_id: ComponentId(uuid::Uuid::new_v4()),
            position: Point::new(2.0, 2.0),
            rotation: Rotation(0.0),
            mirrored: false,
            refdes: "R1".to_string(),
            value: "10k".to_string(),
            locked: false,
        });
        let path = std::env::temp_dir().join("tracecraft-kicad-export.kicad_pcb");
        export_kicad_pcb(&board, path.to_str().unwrap()).unwrap();
        let text = std::fs::read_to_string(&path).unwrap();
        assert!(text.contains("(segment (start 0 0) (end 5 0)"));
        assert!(text.contains("(segment (start 5 0) (end 5 5)"));
        assert!(text.contains("(property \"Value\" 10k)"));
        let _ = std::fs::remove_file(path);
    }
}
