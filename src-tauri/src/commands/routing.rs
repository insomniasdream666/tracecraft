use tauri::{State, Emitter};
use crate::app_state::AppState;
use crate::engine::autorouter::{self, AStarRouter, RouterConfig};
use crate::engine::fanout::fanout_footprint;
use crate::model::id::*;
use crate::model::geometry::Point;
use serde::{Deserialize, Serialize};
use std::sync::atomic::Ordering;

#[derive(Debug, Serialize, Deserialize)]
pub struct RouteRequest {
    pub start_x: f64,
    pub start_y: f64,
    pub end_x: f64,
    pub end_y: f64,
    pub start_layer: Option<u8>,
    pub end_layer: Option<u8>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct RouteResponse {
    pub success: bool,
    pub traces: Vec<SerializedTrace>,
    pub path_length: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SerializedTrace {
    pub layer: u8,
    pub points: Vec<[f64; 2]>,
    pub width: f64,
}

#[tauri::command]
pub fn autoroute_net(
    state: State<AppState>,
    request: RouteRequest,
) -> Result<RouteResponse, String> {
    let board = state.board.read();
    let cfg = RouterConfig::default();
    let router = AStarRouter::new(cfg);

    let start = Point::new(request.start_x, request.start_y);
    let end = Point::new(request.end_x, request.end_y);
    let start_layer = LayerId(request.start_layer.unwrap_or(1));
    let end_layer = LayerId(request.end_layer.unwrap_or(2));

    let result = router.route_net(&board, start, end, start_layer, end_layer, None);

    let traces: Vec<SerializedTrace> = result
        .traces
        .into_iter()
        .map(|t| SerializedTrace {
            layer: t.layer.0,
            points: t.points.into_iter().map(|p| [p.x, p.y]).collect(),
            width: t.width,
        })
        .collect();

    Ok(RouteResponse {
        success: result.success,
        traces,
        path_length: result.path_length,
    })
}

#[tauri::command]
pub fn autoroute_all(
    state: State<AppState>,
) -> Result<Vec<RouteResponse>, String> {
    let board = state.board.read();
    let config = RouterConfig::default();
    state.routing_cancelled.store(false, Ordering::SeqCst);
    let handle = state.app_handle.read().clone();
    let results = autorouter::route_all_nets(&board, config, |current, total| {
        if state.routing_cancelled.load(Ordering::SeqCst) {
            return true;
        }
        if let Some(ref h) = handle {
            let _ = h.emit("routing-progress", serde_json::json!({
                "current": current,
                "total": total,
            }));
        }
        false
    });
    let responses: Vec<RouteResponse> = results
        .into_iter()
        .map(|(_, r)| RouteResponse {
            success: r.success,
            traces: r
                .traces
                .into_iter()
                .map(|t| SerializedTrace {
                    layer: t.layer.0,
                    points: t.points.into_iter().map(|p| [p.x, p.y]).collect(),
                    width: t.width,
                })
                .collect(),
            path_length: r.path_length,
        })
        .collect();
    if let Some(ref h) = handle {
        let _ = h.emit("routing-done", serde_json::json!({ "total": responses.len() }));
    }
    Ok(responses)
}

#[tauri::command]
pub fn cancel_routing(
    state: State<AppState>,
) -> Result<(), String> {
    state.routing_cancelled.store(true, Ordering::SeqCst);
    Ok(())
}

#[tauri::command]
pub fn fanout_component(
    state: State<AppState>,
    footprint_id: usize,
) -> Result<Vec<usize>, String> {
    let mut board = state.board.write();
    let ids = fanout_footprint(&mut board, FootprintId(footprint_id))?;
    let result: Vec<usize> = ids.into_iter().map(|id| id.0).collect();
    Ok(result)
}
