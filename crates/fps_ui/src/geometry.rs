//! # Geometry Module
//!
//! Provides geometric primitives for UI positioning and sizing.
//! Uses floating-point coordinates for smooth scaling and fractional positioning.

use glam::IVec2;

/// Represents a point in 2D integer space.
pub type Point = IVec2;

/// Represents a rectangular area in floating-point coordinates.
///
/// Rectangles are defined by their top-left corner `(x, y)` and dimensions
/// `(w, h)`. Floating-point coordinates allow for smooth scaling and sub-pixel
/// positioning, which is converted to integer pixels during rendering.
///
/// # Coordinate System
///
/// - Origin `(0, 0)` is at the top-left
/// - X increases to the right
/// - Y increases downward
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Rect {
    pub x: f64,
    pub y: f64,
    pub w: f64,
    pub h: f64,
}

impl Rect {
    /// Creates a new rectangle.
    ///
    /// # Arguments
    ///
    /// * `x` - X coordinate of the top-left corner
    /// * `y` - Y coordinate of the top-left corner
    /// * `w` - Width of the rectangle
    /// * `h` - Height of the rectangle
    pub fn new(x: f64, y: f64, w: f64, h: f64) -> Self {
        Rect { x, y, w, h }
    }

    /// Checks if a point is within the rectangle's bounds.
    ///
    /// Used for hit-testing (e.g., checking if mouse cursor is over a component).
    ///
    /// # Arguments
    ///
    /// * `px` - X coordinate of the point
    /// * `py` - Y coordinate of the point
    ///
    /// # Returns
    ///
    /// `true` if the point is inside the rectangle, `false` otherwise.
    pub fn contains(&self, px: f64, py: f64) -> bool {
        px >= self.x as f64
            && py >= self.y as f64
            && px < (self.x + self.w) as f64
            && py < (self.y + self.h) as f64
    }

    /// Returns the top-left corner as an integer point.
    ///
    /// Coordinates are truncated (floored) to integers.
    pub fn top_left(&self) -> Point {
        IVec2::new(self.x as i32, self.y as i32)
    }

    /// Returns the bottom-right corner as an integer point (exclusive).
    ///
    /// The bottom-right corner is exclusive, meaning it's one pixel beyond
    /// the actual rectangle bounds.
    pub fn bottom_right(&self) -> Point {
        IVec2::new((self.x + self.w) as i32, (self.y + self.h) as i32)
    }

    /// Converts to an integer rectangle for pixel-perfect rendering.
    ///
    /// Top-left coordinates are truncated, while dimensions are rounded.
    /// This ensures consistent pixel alignment during rendering.
    ///
    /// # Returns
    ///
    /// An `IntRect` with pixel-snapped coordinates.
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
///
/// Used for actual pixel rendering where fractional coordinates are not meaningful.
/// Width and height are stored as `u32` to ensure non-negative dimensions.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct IntRect {
    pub x: i32,
    pub y: i32,
    pub w: u32,
    pub h: u32,
}
