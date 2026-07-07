use tauri::State;
use serde::Serialize;
use crate::app_state::AppState;
use crate::engine::netlist::extract_netlist;
use crate::engine::ratsnest::{compute_ratsnest, RatsnestLine};
use crate::model::id::*;
use crate::model::geometry::*;
use crate::model::pcb::*;
use crate::model::schematic::NetLabel;
use crate::model::library::ComponentId;
use crate::undo::command::*;
use crate::engine::impedance;
use crate::engine::tuning;
use crate::engine::stitching;

#[derive(Debug, Clone, Serialize)]
pub struct Model3DData {
    pub vertices: Vec<f64>,
    pub normals: Vec<f64>,
}

#[tauri::command]
pub fn undo_pcb(state: State<AppState>) -> Result<(), String> {
    let mut board = state.board.write();
    let mut schematics = state.schematics.write();
    let mut history = state.command_history.write();
    if schematics.is_empty() {
        let mut dummy = crate::model::schematic::SchematicSheet::default();
        history.undo(&mut board, &mut dummy)
    } else {
        let mut first_sheet = schematics.values_mut().next().unwrap();
        history.undo(&mut board, &mut first_sheet)
    }
}

#[tauri::command]
pub fn redo_pcb(state: State<AppState>) -> Result<(), String> {
    let mut board = state.board.write();
    let mut schematics = state.schematics.write();
    let mut history = state.command_history.write();
    if schematics.is_empty() {
        let mut dummy = crate::model::schematic::SchematicSheet::default();
        history.redo(&mut board, &mut dummy)
    } else {
        let mut first_sheet = schematics.values_mut().next().unwrap();
        history.redo(&mut board, &mut first_sheet)
    }
}

#[tauri::command]
pub fn get_3d_model_data(path: String) -> Result<Model3DData, String> {
    let data = std::fs::read(&path).map_err(|e| format!("Failed to read 3D model file: {}", e))?;
    let ext = std::path::Path::new(&path)
        .extension()
        .and_then(|e| e.to_str())
        .unwrap_or("")
        .to_lowercase();
    match ext.as_str() {
        "stl" => parse_stl(&data),
        "obj" => parse_obj(&data),
        _ => Err(format!("Unsupported 3D model format: .{}", ext)),
    }
}

fn parse_stl(data: &[u8]) -> Result<Model3DData, String> {
    if let Ok(text) = std::str::from_utf8(data) {
        let lower = text.to_lowercase();
        if lower.trim_start().starts_with("solid") && text.contains("vertex") {
            return parse_ascii_stl(text);
        }
    }
    parse_binary_stl(data)
}

fn parse_ascii_stl(text: &str) -> Result<Model3DData, String> {
    let mut vertices: Vec<f64> = Vec::new();
    let mut normals: Vec<f64> = Vec::new();
    let mut current_normal: [f64; 3] = [0.0, 1.0, 0.0];

    for line in text.lines() {
        let trimmed = line.trim();
        if trimmed.is_empty() { continue; }
        let parts: Vec<&str> = trimmed.split_whitespace().collect();
        if parts.is_empty() { continue; }
        match parts[0] {
            "facet" if parts.len() >= 5 && parts[1] == "normal" => {
                current_normal = [
                    parts[2].parse().unwrap_or(0.0),
                    parts[3].parse().unwrap_or(0.0),
                    parts[4].parse().unwrap_or(0.0),
                ];
            }
            "vertex" if parts.len() >= 4 => {
                let x = parts[1].parse::<f64>().unwrap_or(0.0);
                let y = parts[2].parse::<f64>().unwrap_or(0.0);
                let z = parts[3].parse::<f64>().unwrap_or(0.0);
                vertices.push(x); vertices.push(y); vertices.push(z);
                normals.push(current_normal[0]);
                normals.push(current_normal[1]);
                normals.push(current_normal[2]);
            }
            _ => {}
        }
    }
    if vertices.is_empty() {
        return Err("No vertices found in ASCII STL file".to_string());
    }
    Ok(Model3DData { vertices, normals })
}

fn parse_binary_stl(data: &[u8]) -> Result<Model3DData, String> {
    if data.len() < 84 {
        return Err("File too small for binary STL".to_string());
    }
    let triangle_count = u32::from_le_bytes(
        [data[80], data[81], data[82], data[83]],
    ) as usize;
    if data.len() < 84 + triangle_count * 50 {
        return Err("File truncated or invalid binary STL".to_string());
    }
    let mut vertices = Vec::with_capacity(triangle_count * 9);
    let mut normals = Vec::with_capacity(triangle_count * 9);
    for i in 0..triangle_count {
        let off = 84 + i * 50;
        let nx = f32::from_le_bytes([data[off], data[off+1], data[off+2], data[off+3]]) as f64;
        let ny = f32::from_le_bytes([data[off+4], data[off+5], data[off+6], data[off+7]]) as f64;
        let nz = f32::from_le_bytes([data[off+8], data[off+9], data[off+10], data[off+11]]) as f64;
        for j in 0..3 {
            let vo = off + 12 + j * 12;
            let vx = f32::from_le_bytes([data[vo], data[vo+1], data[vo+2], data[vo+3]]) as f64;
            let vy = f32::from_le_bytes([data[vo+4], data[vo+5], data[vo+6], data[vo+7]]) as f64;
            let vz = f32::from_le_bytes([data[vo+8], data[vo+9], data[vo+10], data[vo+11]]) as f64;
            vertices.push(vx); vertices.push(vy); vertices.push(vz);
            normals.push(nx); normals.push(ny); normals.push(nz);
        }
    }
    Ok(Model3DData { vertices, normals })
}

fn parse_obj(data: &[u8]) -> Result<Model3DData, String> {
    let text = std::str::from_utf8(data).map_err(|e| format!("Invalid UTF-8 in OBJ file: {}", e))?;
    let mut raw_vertices: Vec<[f64; 3]> = Vec::new();
    let mut raw_normals: Vec<[f64; 3]> = Vec::new();
    struct FaceCorner { vi: usize, ni: Option<usize> }
    let mut corners: Vec<FaceCorner> = Vec::new();
    let mut face_sizes: Vec<usize> = Vec::new();

    for line in text.lines() {
        let trimmed = line.trim();
        if trimmed.is_empty() || trimmed.starts_with('#') { continue; }
        let parts: Vec<&str> = trimmed.split_whitespace().collect();
        if parts.is_empty() { continue; }
        match parts[0] {
            "v" if parts.len() >= 4 => {
                raw_vertices.push([
                    parts[1].parse().unwrap_or(0.0),
                    parts[2].parse().unwrap_or(0.0),
                    parts[3].parse().unwrap_or(0.0),
                ]);
            }
            "vn" if parts.len() >= 4 => {
                raw_normals.push([
                    parts[1].parse().unwrap_or(0.0),
                    parts[2].parse().unwrap_or(0.0),
                    parts[3].parse().unwrap_or(0.0),
                ]);
            }
            "f" if parts.len() >= 4 => {
                let start = corners.len();
                for part in &parts[1..] {
                    let idxs: Vec<&str> = part.split('/').collect();
                    let vi = idxs[0].parse::<usize>().unwrap_or(0);
                    if vi == 0 { continue; }
                    let ni = if idxs.len() >= 3 && !idxs[2].is_empty() {
                        idxs[2].parse::<usize>().ok().map(|n| n - 1)
                    } else {
                        None
                    };
                    corners.push(FaceCorner { vi: vi - 1, ni });
                }
                let cnt = corners.len() - start;
                if cnt >= 3 { face_sizes.push(cnt); }
                else { corners.truncate(start); }
            }
            _ => {}
        }
    }
    if face_sizes.is_empty() {
        return Err("No valid faces found in OBJ file".to_string());
    }
    let mut vertices: Vec<f64> = Vec::new();
    let mut normals: Vec<f64> = Vec::new();
    let mut corner_idx = 0usize;
    let has_any_normals = !raw_normals.is_empty();
    let mut use_normals = false;

    for &size in &face_sizes {
        for t in 1..size - 1 {
            let tri = [0usize, t, t + 1];
            for &c in &tri {
                let fc = &corners[corner_idx + c];
                if fc.vi < raw_vertices.len() {
                    let v = raw_vertices[fc.vi];
                    vertices.push(v[0]); vertices.push(v[1]); vertices.push(v[2]);
                }
                if has_any_normals {
                    if let Some(ni) = fc.ni {
                        if ni < raw_normals.len() {
                            let n = raw_normals[ni];
                            normals.push(n[0]); normals.push(n[1]); normals.push(n[2]);
                            use_normals = true;
                        }
                    }
                }
            }
        }
        corner_idx += size;
    }
    // If normals were missing for some faces, discard and compute flat normals
    if !use_normals || normals.len() != vertices.len() {
        normals.clear();
    }
    // Compute flat normals if none were loaded
    if normals.is_empty() && vertices.len() >= 9 {
        for i in 0..vertices.len() / 9 {
            let base = i * 9;
            let v0 = [vertices[base], vertices[base+1], vertices[base+2]];
            let v1 = [vertices[base+3], vertices[base+4], vertices[base+5]];
            let v2 = [vertices[base+6], vertices[base+7], vertices[base+8]];
            let ux = v1[0] - v0[0]; let uy = v1[1] - v0[1]; let uz = v1[2] - v0[2];
            let vx = v2[0] - v0[0]; let vy = v2[1] - v0[1]; let vz = v2[2] - v0[2];
            let nx = uy * vz - uz * vy;
            let ny = uz * vx - ux * vz;
            let nz = ux * vy - uy * vx;
            let len = (nx*nx + ny*ny + nz*nz).sqrt();
            let nlen = if len > 0.0 { len } else { 1.0 };
            for _ in 0..3 {
                normals.push(nx / nlen);
                normals.push(ny / nlen);
                normals.push(nz / nlen);
            }
        }
    }
    Ok(Model3DData { vertices, normals })
}

#[tauri::command]
pub fn get_board_state(state: State<AppState>) -> Result<PcbBoard, String> {
    Ok(state.board.read().clone())
}

#[tauri::command]
pub fn place_footprint(
    state: State<AppState>,
    component_id: ComponentId,
    position: Point,
    rotation: Rotation,
) -> Result<FootprintId, String> {
    let mut board = state.board.write();
    let cmd = PlaceFootprint {
        component_id,
        position,
        rotation,
        result_id: None,
    };
    let mut boxed: Box<dyn Command> = Box::new(cmd);
    let mut dummy = crate::model::schematic::SchematicSheet::default();
    boxed.execute(&mut board, &mut dummy)?;
    let id = FootprintId(board.footprints.len() - 1);
    Ok(id)
}

#[tauri::command]
pub fn route_trace(
    state: State<AppState>,
    net_id: Option<NetId>,
    layer: LayerId,
    width: f64,
    points: Vec<Point>,
) -> Result<TraceId, String> {
    let mut board = state.board.write();

    // Use the push-and-shove engine to resolve conflicts with existing traces
    let adjusted_points = crate::engine::shove::resolve_conflicts(
        &points, width, board.design_rules.min_clearance,
        &mut board, layer, net_id,
    );

    let trace = Trace {
        net_id,
        layer,
        width,
        points: adjusted_points,
    };
    let id = TraceId(board.traces.insert(trace));
    Ok(id)
}

#[tauri::command]
pub fn add_via(
    state: State<AppState>,
    position: Point,
    diameter: f64,
    drill: f64,
) -> Result<ViaId, String> {
    let mut board = state.board.write();
    let via = Via {
        position,
        start_layer: LayerId(1),
        end_layer: LayerId(2),
        diameter,
        drill,
        net_id: None,
    };
    let id = ViaId(board.vias.insert(via));
    Ok(id)
}

#[tauri::command]
pub fn set_board_outline(
    state: State<AppState>,
    shape: Vec<Vec<Point>>,
    cutouts: Vec<Vec<Point>>,
) -> Result<(), String> {
    let mut board = state.board.write();
    board.outline = Some(BoardOutline { shape, cutouts });
    Ok(())
}

#[tauri::command]
pub fn sync_netlist(state: State<AppState>) -> Result<(), String> {
    let schematics = state.schematics.read();
    let mut next_id = 0usize;

    let mut all_nets = Vec::new();
    for sheet in schematics.values() {
        let netlist = extract_netlist(sheet, &mut next_id);
        all_nets.extend(netlist.nets);
    }
    drop(schematics);

    let mut board = state.board.write();
    board.netlist = crate::model::netlist::Netlist { nets: all_nets };
    Ok(())
}

#[tauri::command]
pub fn sync_pcb_to_schematic(state: State<AppState>) -> Result<(), String> {
    let board = state.board.read();
    let nets = &board.netlist.nets;
    let mut schematics = state.schematics.write();
    for (_, sheet) in schematics.iter_mut() {
        for net in nets {
            let has_matching_label = sheet.net_labels.iter().any(|(_, nl)| nl.name == net.name);
            if !has_matching_label && net.name.starts_with("N$") {
                // Skip auto-generated net names, only sync user-named nets
                continue;
            }
            if !has_matching_label && !net.connections.is_empty() {
                let pos = net.connections.first().map(|c| {
                    board.footprints.iter().find_map(|(_, fp)| {
                        if fp.refdes == c.refdes { Some(fp.position) } else { None }
                    }).unwrap_or_default()
                }).unwrap_or_default();
                sheet.net_labels.insert(NetLabel {
                    name: net.name.clone(),
                    position: pos,
                });
            }
        }
    }
    Ok(())
}

#[tauri::command]
pub fn get_ratsnest(state: State<AppState>) -> Result<Vec<RatsnestLine>, String> {
    let board = state.board.read();
    let lines = compute_ratsnest(&board);
    Ok(lines)
}

#[tauri::command]
pub fn delete_pcb_element(
    state: State<AppState>,
    element_type: String,
    element_id: usize,
) -> Result<(), String> {
    let mut board = state.board.write();
    match element_type.as_str() {
        "footprint" => { board.footprints.remove(element_id); }
        "trace" => { board.traces.remove(element_id); }
        "via" => { board.vias.remove(element_id); }
        "zone" => { board.zones.remove(element_id); }
        _ => return Err(format!("Unknown PCB element type: {}", element_type)),
    }
    Ok(())
}

#[derive(Debug, Clone, Serialize)]
pub struct BoardProperties {
    pub has_outline: bool,
    pub width: f64,
    pub height: f64,
    pub layer_count: usize,
    pub layer_names: Vec<String>,
}

#[tauri::command]
pub fn get_board_properties(state: State<AppState>) -> Result<BoardProperties, String> {
    let board = state.board.read();
    let (width, height) = match &board.outline {
        Some(outline) if !outline.shape.is_empty() && outline.shape[0].len() >= 2 => {
            let pts = &outline.shape[0];
            let xs: Vec<f64> = pts.iter().map(|p| p.x).collect();
            let ys: Vec<f64> = pts.iter().map(|p| p.y).collect();
            let w = xs.iter().cloned().fold(f64::NEG_INFINITY, f64::max) - xs.iter().cloned().fold(f64::INFINITY, f64::min);
            let h = ys.iter().cloned().fold(f64::NEG_INFINITY, f64::max) - ys.iter().cloned().fold(f64::INFINITY, f64::min);
            (w.abs(), h.abs())
        }
        _ => (100.0, 80.0),
    };
    Ok(BoardProperties {
        has_outline: board.outline.is_some(),
        width,
        height,
        layer_count: board.layers.layers.len(),
        layer_names: board.layers.layers.iter().map(|l| l.name.clone()).collect(),
    })
}

#[tauri::command]
pub fn get_stackup(state: State<AppState>) -> Result<LayerStack, String> {
    Ok(state.board.read().layers.clone())
}

#[tauri::command]
pub fn set_stackup(state: State<AppState>, stackup: LayerStack) -> Result<(), String> {
    state.board.write().layers = stackup;
    Ok(())
}

#[tauri::command]
pub fn compute_impedance(
    state: State<AppState>,
    target_impedance: f64,
    layer_index: usize,
) -> Result<ImpedanceResult, String> {
    let board = state.board.read();
    let t = impedance::copper_thickness_mm(board.layers.copper_weight_oz);
    Ok(impedance::microstrip_impedance(target_impedance, &board.layers, layer_index, t))
}

#[tauri::command]
pub fn route_bus(
    state: State<AppState>,
    net_ids: Vec<NetId>,
    layer: LayerId,
    start_points: Vec<Point>,
    end_points: Vec<Point>,
    width: f64,
    spacing: f64,
) -> Result<Vec<TraceId>, String> {
    if net_ids.len() != start_points.len() || net_ids.len() != end_points.len() {
        return Err("Mismatched net/point count for bus routing".to_string());
    }
    let mut board = state.board.write();
    let mut ids = Vec::new();
    let count = net_ids.len();
    let total_width = (count - 1) as f64 * spacing;
    let start_offset = -total_width / 2.0;

    for i in 0..count {
        let dx = end_points[i].x - start_points[i].x;
        let dy = end_points[i].y - start_points[i].y;
        let len = (dx * dx + dy * dy).sqrt();
        let offset = start_offset + i as f64 * spacing;
        let (nx, ny) = if len > 1e-6 { (-dy / len * offset, dx / len * offset) } else { (0.0, 0.0) };

        let adjusted_start = Point::new(start_points[i].x + nx, start_points[i].y + ny);
        let adjusted_end = Point::new(end_points[i].x + nx, end_points[i].y + ny);

        let trace = Trace {
            net_id: Some(net_ids[i]),
            layer,
            width,
            points: vec![adjusted_start, adjusted_end],
        };
        ids.push(TraceId(board.traces.insert(trace)));
    }
    Ok(ids)
}

#[tauri::command]
pub fn apply_length_tuning(
    state: State<AppState>,
    trace_index: usize,
    segment_index: usize,
    extra_length: f64,
) -> Result<(), String> {
    let mut board = state.board.write();
    if let Some(t) = board.traces.get_mut(trace_index) {
        let tuned = tuning::apply_tuning(t, segment_index, extra_length);
        *t = tuned;
    }
    Ok(())
}

#[tauri::command]
pub fn stitch_vias(
    state: State<AppState>,
    start_x: f64, start_y: f64,
    end_x: f64, end_y: f64,
    net_id: usize,
    spacing: f64,
) -> Result<Vec<ViaId>, String> {
    let (via_diameter, via_drill) = {
        let board = state.board.read();
        (board.design_rules.default_via_diameter, board.design_rules.default_via_drill)
    };
    let mut board = state.board.write();
    let start = Point::new(start_x, start_y);
    let end = Point::new(end_x, end_y);
    let ids = stitching::stitch_via_line(
        &mut board, start, end, NetId(net_id), spacing,
        via_diameter, via_drill,
        LayerId(1), LayerId(2),
    );
    Ok(ids)
}

#[tauri::command]
pub fn stitch_via_shield(
    state: State<AppState>,
    trace_index: usize,
    offset: f64,
    spacing: f64,
) -> Result<Vec<ViaId>, String> {
    let (trace, via_diameter, via_drill) = {
        let board = state.board.read();
        let t = board.traces.get(trace_index)
            .ok_or("Trace not found")?
            .clone();
        (t, board.design_rules.default_via_diameter, board.design_rules.default_via_drill)
    };
    let mut board = state.board.write();
    let net_id = trace.net_id.unwrap_or(NetId(0));
    let ids = stitching::stitch_via_shield(
        &mut board, &trace.points, net_id, spacing, offset,
        via_diameter, via_drill,
        LayerId(1), LayerId(2),
    );
    Ok(ids)
}

#[tauri::command]
pub fn set_board_size(state: State<AppState>, width: f64, height: f64) -> Result<(), String> {
    let mut board = state.board.write();
    let half_w = width / 2.0;
    let half_h = height / 2.0;
    board.outline = Some(BoardOutline {
        shape: vec![vec![
            Point::new(-half_w, -half_h),
            Point::new(half_w, -half_h),
            Point::new(half_w, half_h),
            Point::new(-half_w, half_h),
            Point::new(-half_w, -half_h),
        ]],
        cutouts: vec![],
    });
    Ok(())
}

#[tauri::command]
pub fn create_diff_pair(
    state: State<AppState>,
    name: String,
    p_net: NetId,
    n_net: NetId,
    width: f64,
    gap: f64,
) -> Result<(), String> {
    let mut board = state.board.write();
    board.diff_pairs.push(DifferentialPair { name, p_net, n_net, width, gap });
    Ok(())
}

#[tauri::command]
pub fn list_diff_pairs(state: State<AppState>) -> Result<Vec<DifferentialPair>, String> {
    Ok(state.board.read().diff_pairs.clone())
}

#[tauri::command]
pub fn delete_diff_pair(state: State<AppState>, index: usize) -> Result<(), String> {
    let mut board = state.board.write();
    if index < board.diff_pairs.len() {
        board.diff_pairs.remove(index);
    }
    Ok(())
}

#[tauri::command]
pub fn route_diff_pair(
    state: State<AppState>,
    pair_index: usize,
    layer: LayerId,
    start_p: Point,
    start_n: Point,
    end_p: Point,
    end_n: Point,
) -> Result<Vec<TraceId>, String> {
    let mut board = state.board.write();
    let pair = board.diff_pairs.get(pair_index)
        .ok_or_else(|| "Differential pair not found".to_string())?
        .clone();

    let gap = pair.gap;
    let width = pair.width;

    // Route both traces in parallel by taking the midpoint path and offsetting
    let mid_points = vec![
        Point::new((start_p.x + start_n.x) / 2.0, (start_p.y + start_n.y) / 2.0),
        Point::new((end_p.x + end_n.x) / 2.0, (end_p.y + end_n.y) / 2.0),
    ];

    let dx = mid_points[1].x - mid_points[0].x;
    let dy = mid_points[1].y - mid_points[0].y;
    let len = (dx * dx + dy * dy).sqrt();
    let (nx, ny) = if len > 1e-6 { (-dy / len * gap / 2.0, dx / len * gap / 2.0) } else { (0.0, 0.0) };

    let p_trace = Trace {
        net_id: Some(pair.p_net),
        layer,
        width,
        points: vec![
            Point::new(mid_points[0].x + nx, mid_points[0].y + ny),
            Point::new(mid_points[1].x + nx, mid_points[1].y + ny),
        ],
    };
    let n_trace = Trace {
        net_id: Some(pair.n_net),
        layer,
        width,
        points: vec![
            Point::new(mid_points[0].x - nx, mid_points[0].y - ny),
            Point::new(mid_points[1].x - nx, mid_points[1].y - ny),
        ],
    };

    let p_id = TraceId(board.traces.insert(p_trace));
    let n_id = TraceId(board.traces.insert(n_trace));
    Ok(vec![p_id, n_id])
}

#[tauri::command]
pub fn draw_zone(
    state: State<AppState>,
    layer: LayerId,
    outline: Vec<Point>,
) -> Result<ZoneId, String> {
    let mut board = state.board.write();
    let zone = Zone {
        net_id: None,
        layer,
        outline,
        clearance: 0.2,
        thermal_relief: true,
    };
    let id = ZoneId(board.zones.insert(zone));
    Ok(id)
}
