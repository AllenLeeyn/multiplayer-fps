use glam::IVec2;

/// Represents a point in 2D space.
pub type Point = IVec2;

/// Represents a rectangular area, defined by its top-left corner.
/// Dimensions are stored as f64 for scaling.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Rect {
    pub x: f64,
    pub y: f64,
    pub w: f64,
    pub h: f64,
}

impl Rect {
    /// Creates a new standard Rect instance.
    pub fn new(x: f64, y: f64, w: f64, h: f64) -> Self {
        Rect { x, y, w, h }
    }

    /// Checks if a given coordinate (e.g., mouse position) falls within these bounds.
    pub fn contains(&self, px: f64, py: f64) -> bool {
        px >= self.x as f64
            && py >= self.y as f64
            && px < (self.x + self.w) as f64
            && py < (self.y + self.h) as f64
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

    /// Converts the floating-point Rect to an IntRect, snapping coordinates to pixels.
    /// This is used immediately prior to drawing for pixel-perfect rendering.
    pub fn to_int_rect(&self) -> IntRect {
        // Top-left is truncated (as i32)
        let x_int = self.x.trunc() as i32;
        let y_int = self.y.trunc() as i32;

        // Bottom-right is rounded, then difference calculates the integer width/height.
        let x_end = (self.x + self.w).round() as i32;
        let y_end = (self.y + self.h).round() as i32;

        let w_int = x_end - x_int;
        let h_int = y_end - y_int;

        IntRect {
            x: x_int,
            y: y_int,
            w: w_int.max(0) as u32,
            h: h_int.max(0) as u32,
        }
    }
}

/// Represents an integer-based, pixel-snapped rectangle.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct IntRect {
    pub x: i32,
    pub y: i32,
    pub w: u32,
    pub h: u32,
}
