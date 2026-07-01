use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Default, PartialEq, Serialize, Deserialize)]
pub struct Point {
    pub x: f64,
    pub y: f64,
}

impl Point {
    pub fn new(x: f64, y: f64) -> Self {
        Self { x, y }
    }

    pub fn distance(&self, other: &Point) -> f64 {
        ((self.x - other.x).powi(2) + (self.y - other.y).powi(2)).sqrt()
    }

    pub fn rotate(&self, center: &Point, angle_deg: f64) -> Point {
        let rad = angle_deg.to_radians();
        let cos = rad.cos();
        let sin = rad.sin();
        let dx = self.x - center.x;
        let dy = self.y - center.y;
        Point::new(
            center.x + dx * cos - dy * sin,
            center.y + dx * sin + dy * cos,
        )
    }

    pub fn snap(&self, grid: f64) -> Point {
        Point::new(
            (self.x / grid).round() * grid,
            (self.y / grid).round() * grid,
        )
    }

    pub fn midpoint(&self, other: &Point) -> Point {
        Point::new((self.x + other.x) / 2.0, (self.y + other.y) / 2.0)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct Size {
    pub width: f64,
    pub height: f64,
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct Rect {
    pub x: f64,
    pub y: f64,
    pub width: f64,
    pub height: f64,
}

impl Rect {
    pub fn contains(&self, point: &Point) -> bool {
        point.x >= self.x
            && point.x <= self.x + self.width
            && point.y >= self.y
            && point.y <= self.y + self.height
    }

    pub fn from_points(points: &[Point]) -> Option<Rect> {
        if points.is_empty() {
            return None;
        }
        let min_x = points.iter().map(|p| p.x).fold(f64::INFINITY, f64::min);
        let max_x = points.iter().map(|p| p.x).fold(f64::NEG_INFINITY, f64::max);
        let min_y = points.iter().map(|p| p.y).fold(f64::INFINITY, f64::min);
        let max_y = points.iter().map(|p| p.y).fold(f64::NEG_INFINITY, f64::max);
        Some(Rect {
            x: min_x,
            y: min_y,
            width: max_x - min_x,
            height: max_y - min_y,
        })
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
#[serde(transparent)]
pub struct Rotation(pub f64);

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_point_new() {
        let p = Point::new(3.0, 4.0);
        assert_eq!(p.x, 3.0);
        assert_eq!(p.y, 4.0);
    }

    #[test]
    fn test_point_distance() {
        let a = Point::new(0.0, 0.0);
        let b = Point::new(3.0, 4.0);
        assert!((a.distance(&b) - 5.0).abs() < 1e-10);
    }

    #[test]
    fn test_point_midpoint() {
        let a = Point::new(0.0, 0.0);
        let b = Point::new(10.0, 20.0);
        let m = a.midpoint(&b);
        assert!((m.x - 5.0).abs() < 1e-10);
        assert!((m.y - 10.0).abs() < 1e-10);
    }

    #[test]
    fn test_point_snap() {
        let p = Point::new(3.7, 8.2);
        let snapped = p.snap(2.54);
        assert!((snapped.x - 2.54).abs() < 1e-6);
        assert!((snapped.y - 7.62).abs() < 1e-6);
    }

    #[test]
    fn test_point_rotate_90() {
        let center = Point::new(0.0, 0.0);
        let p = Point::new(1.0, 0.0);
        let r = p.rotate(&center, 90.0);
        assert!((r.x - 0.0).abs() < 1e-10);
        assert!((r.y - 1.0).abs() < 1e-10);
    }

    #[test]
    fn test_rect_contains() {
        let r = Rect { x: 0.0, y: 0.0, width: 10.0, height: 10.0 };
        assert!(r.contains(&Point::new(5.0, 5.0)));
        assert!(!r.contains(&Point::new(15.0, 5.0)));
        assert!(!r.contains(&Point::new(-1.0, 5.0)));
    }

    #[test]
    fn test_rect_from_points() {
        let pts = vec![Point::new(3.0, 7.0), Point::new(10.0, 2.0), Point::new(5.0, 5.0)];
        let r = Rect::from_points(&pts).unwrap();
        assert!((r.x - 3.0).abs() < 1e-10);
        assert!((r.y - 2.0).abs() < 1e-10);
        assert!((r.width - 7.0).abs() < 1e-10);
        assert!((r.height - 5.0).abs() < 1e-10);
    }

    #[test]
    fn test_rect_from_empty() {
        assert!(Rect::from_points(&[]).is_none());
    }
}
