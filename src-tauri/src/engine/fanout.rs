use crate::model::pcb::*;
use crate::model::geometry::*;
use crate::model::id::*;

/// Fan-out a footprint: place short escape traces from each pad position
/// to a nearby via, clearing room for routing.
pub fn fanout_footprint(board: &mut PcbBoard, footprint_id: FootprintId) -> Result<Vec<TraceId>, String> {
    let fp = board.footprints.get(footprint_id.0)
        .ok_or("Footprint not found")?;

    let center = fp.position;
    let mut trace_ids = Vec::new();

    // Generate 4 escape traces in cardinal directions as a simple fan-out
    let directions = [
        (0.0, -1.0),  // up
        (1.0, 0.0),   // right
        (0.0, 1.0),   // down
        (-1.0, 0.0),  // left
    ];

    for (_i, &(dx, dy)) in directions.iter().enumerate() {
        let start = Point::new(
            center.x + dx * 3.0,
            center.y + dy * 3.0,
        );
        let end = Point::new(
            center.x + dx * 8.0,
            center.y + dy * 8.0,
        );

        // Place escape trace
        let trace = Trace {
            net_id: None,
            layer: LayerId(1),
            width: 0.25,
            points: vec![start, end],
        };
        let id = board.traces.insert(trace);
        trace_ids.push(TraceId(id));

        // Place via at the end
        let via = Via {
            position: end,
            start_layer: LayerId(1),
            end_layer: LayerId(2),
            diameter: 0.6,
            drill: 0.3,
            net_id: None,
        };
        board.vias.insert(via);
    }

    Ok(trace_ids)
}
