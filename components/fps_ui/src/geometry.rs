use glam::IVec2;

/// Represents a point in 2D space.
pub type Point = IVec2;

/// Represents a rectangular area, defined by its top-left corner, dimensions,
/// and an optional corner radius.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Rect {
    pub x: i32,
    pub y: i32,
    pub w: i32,
    pub h: i32,
    /// The radius applied to all four corners. If 0, the corners are sharp.
    /// Clamped internally to prevent overlapping circles (max is min(w, h) / 2).
    pub radius: i32,
}

impl Rect {
    /// Creates a new standard Rect instance (no rounded corners).
    pub fn new(x: i32, y: i32, w: i32, h: i32) -> Self {
        Rect {
            x,
            y,
            w,
            h,
            radius: 0,
        }
    }

    /// Creates a new Rect instance with specified rounded corners.
    pub fn new_rounded(x: i32, y: i32, w: i32, h: i32, radius: i32) -> Self {
        // Ensure the radius doesn't exceed half the smallest dimension.
        let max_radius = std::cmp::min(w, h) / 2;
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
    /// NOTE: This check does NOT account for the rounded corners.
    /// For UI components, this is often sufficient for hit-testing, as the
    /// invisible corner area is usually negligible.
    pub fn contains_point(&self, point: Point) -> bool {
        point.x >= self.x
            && point.x < (self.x + self.w)
            && point.y >= self.y
            && point.y < (self.y + self.h)
    }

    /// Returns the top-left corner as a Point.
    pub fn top_left(&self) -> Point {
        IVec2::new(self.x, self.y)
    }

    /// Returns the bottom-right corner as a Point (exclusive).
    pub fn bottom_right(&self) -> Point {
        IVec2::new(self.x + self.w, self.y + self.h)
    }

    // Helper to get the corner radius.
    pub fn corner_radius(&self) -> i32 {
        self.radius
    }
}
