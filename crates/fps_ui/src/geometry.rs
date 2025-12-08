use glam::IVec2;

/// Represents a point in 2D space.
pub type Point = IVec2;

/// Represents a rectangular area, defined by its top-left corner, dimensions,
/// and an optional corner radius.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Rect {
    pub x: f64,
    pub y: f64,
    pub w: f64,
    pub h: f64,
    pub radius: i32,
}

pub type Bounds = Rect;

impl Rect {
    /// Creates a new standard Rect instance (no rounded corners).
    pub fn new(x: f64, y: f64, w: f64, h: f64) -> Self {
        Rect {
            x,
            y,
            w,
            h,
            radius: 0,
        }
    }

    /// Creates a new Rect instance with specified rounded corners.
    pub fn new_rounded(x: f64, y: f64, w: f64, h: f64, radius: i32) -> Self {
        let max_radius = std::cmp::min(w as i32, h as i32) / 2;
        let clamped_radius = std::cmp::min(radius, max_radius);

        Rect {
            x,
            y,
            w,
            h,
            radius: clamped_radius,
        }
    }

    /// Checks if a given point is contained within this rectangle's bounds.
    pub fn contains_point(&self, point: Point) -> bool {
        let px = point.x as f64;
        let py = point.y as f64;
        px >= self.x
            && px < (self.x + self.w)
            && py >= self.y
            && py < (self.y + self.h)
    }

    /// Returns the top-left corner as a Point.
    pub fn top_left(&self) -> Point {
        // FIX: Explicitly cast f64 coordinates to i32 for Point/IVec2.
        IVec2::new(self.x as i32, self.y as i32)
    }

    /// Returns the bottom-right corner as a Point (exclusive).
    pub fn bottom_right(&self) -> Point {
        // FIX: Explicitly cast f64 coordinates to i32 for Point/IVec2.
        // For the exclusive bottom-right edge, casting the sum is correct.
        IVec2::new((self.x + self.w) as i32, (self.y + self.h) as i32)
    }

    // Helper to get the corner radius.
    pub fn corner_radius(&self) -> i32 {
        self.radius
    }
    
    /// Checks if a given coordinate (e.g., mouse position) falls within these bounds.
    pub fn contains(&self, px: f64, py: f64) -> bool {
        px >= self.x as f64
            && py >= self.y as f64
            && px < (self.x + self.w) as f64
            && py < (self.y + self.h) as f64
    }
}
