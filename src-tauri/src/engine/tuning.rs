use crate::model::geometry::Point;
use crate::model::pcb::Trace;

/// Parameters for a serpentine (accordion) tuning section
pub struct SerpentineParams {
    pub amplitude: f64,     // height of each "tooth" in mm
    pub pitch: f64,         // spacing between teeth in mm
    pub segments: usize,    // number of full teeth
}

/// Generate a serpentine tuning section along a trace segment.
/// Returns new points for the segment that form a serpentine pattern of the desired extra length.
pub fn generate_serpentine(
    start: Point,
    end: Point,
    extra_length: f64,
) -> Vec<Point> {
    let dx = end.x - start.x;
    let dy = end.y - start.y;
    let seg_len = (dx * dx + dy * dy).sqrt();

    if seg_len < 1.0 || extra_length < 0.1 {
        return vec![start, end];
    }

    // Unit direction
    let ux = dx / seg_len;
    let uy = dy / seg_len;

    // Perpendicular unit vector
    let nx = -uy;
    let ny = ux;

    // Determine amplitude and pitch based on extra_length
    // Rule of thumb: amplitude ~ 2x trace width, pitch ~ 4x amplitude
    let amplitude = (extra_length * 0.3).max(0.5).min(5.0);
    let pitch = (amplitude * 2.0).max(1.0);

    // Number of full teeth (each tooth adds ~2*amplitude of extra length + pitch along direction)
    // Each tooth adds: 2*sqrt(pitch²/4 + amplitude²) + extra
    let tooth_extra = 2.0 * ((pitch * pitch / 4.0 + amplitude * amplitude).sqrt()) - pitch;
    let mut segments = (extra_length / tooth_extra.max(0.01)).ceil() as usize;
    segments = segments.max(1).min(20);

    let mut points = Vec::new();
    points.push(start);

    let total_extra = segments as f64 * tooth_extra;
    let total_along = (seg_len - total_extra * 0.1).max(seg_len * 0.1); // pull ends together, min 10% of seg_len
    let along_step = total_along / segments as f64;

    for i in 0..segments {
        let base = i as f64 * along_step;
        // Point along segment
        let px = start.x + ux * base;
        let py = start.y + uy * base;

        // Up tooth
        points.push(Point::new(px + nx * amplitude * 0.5, py + ny * amplitude * 0.5));
        // Forward + down
        let fx = px + ux * along_step;
        let fy = py + uy * along_step;
        points.push(Point::new(fx - nx * amplitude * 0.5, fy - ny * amplitude * 0.5));
    }

    points.push(end);
    points
}

/// Apply serpentine tuning to a trace segment and return the modified trace.
pub fn apply_tuning(
    trace: &Trace,
    segment_index: usize,
    extra_length: f64,
) -> Trace {
    let mut new_trace = trace.clone();

    if segment_index >= trace.points.len().saturating_sub(1) {
        return new_trace;
    }

    let start = trace.points[segment_index];
    let end = trace.points[segment_index + 1];
    let serpentine = generate_serpentine(start, end, extra_length);

    // Replace the segment points, keeping surrounding points
    let mut new_points: Vec<Point> = trace.points[..segment_index].to_vec();
    new_points.extend(serpentine);
    if segment_index + 2 < trace.points.len() {
        new_points.extend_from_slice(&trace.points[segment_index + 2..]);
    }

    new_trace.points = new_points;
    new_trace
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_serpentine_generation() {
        let start = Point::new(0.0, 0.0);
        let end = Point::new(10.0, 0.0);
        let points = generate_serpentine(start, end, 5.0);
        assert!(points.len() > 2);
        // First point should be start
        assert!((points[0].x - 0.0).abs() < 0.01);
        assert!((points[0].y - 0.0).abs() < 0.01);
        // Last point should be end
        assert!((points[points.len() - 1].x - 10.0).abs() < 0.01);
        assert!((points[points.len() - 1].y - 0.0).abs() < 0.01);
    }

    #[test]
    fn test_apply_tuning() {
        let trace = Trace {
            net_id: None,
            layer: crate::model::id::LayerId(1),
            width: 0.25,
            points: vec![
                Point::new(0.0, 0.0),
                Point::new(10.0, 0.0),
                Point::new(20.0, 0.0),
            ],
            attenuation_db_per_m: 0.0,
        };
        let tuned = apply_tuning(&trace, 0, 5.0);
        assert!(tuned.points.len() > trace.points.len());
        assert_eq!(tuned.points[0].x, 0.0);
        assert_eq!(tuned.points.last().unwrap().x, 20.0);
    }
}
