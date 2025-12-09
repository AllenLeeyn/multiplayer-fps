use glam::IVec2;

use super::{AnchorPoint, LayoutMetrics, LengthMode};
/// Represents a point in 2D space.
pub type Point = IVec2;

/// Represents a rectangular area, defined by its top-left corner, dimensions,
/// and an optional corner radius. Dimensions are stored as f64 for scaling.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Rect {
    pub x: f64,
    pub y: f64,
    pub w: f64,
    pub h: f64,
    pub radius: i32,
}

pub type Bounds = Rect;

/// Represents an integer-based, pixel-snapped rectangle.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct IntRect {
    pub x: i32,
    pub y: i32,
    pub w: u32,
    pub h: u32,
    pub radius: i32,
}
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

    // Helper to get the corner radius.
    pub fn corner_radius(&self) -> i32 {
        self.radius
    }

    /// Converts the floating-point Rect to an IntRect, snapping coordinates to pixels.
    /// This is used immediately prior to drawing for pixel-perfect rendering.
    pub fn to_int_rect(&self) -> IntRect {
        // Use rounding or truncation based on rendering preference. Truncation (as i32) 
        // is often used for top-left, and width/height are calculated from that.
        let x_int = self.x as i32;
        let y_int = self.y as i32;
        
        // Calculate width/height based on the difference between integer coordinates
        let w_int = (self.x + self.w).round() as i32 - x_int;
        let h_int = (self.y + self.h).round() as i32 - y_int;

        IntRect {
            x: x_int,
            y: y_int,
            w: w_int.max(0) as u32,
            h: h_int.max(0) as u32,
            radius: self.radius,
        }
    }
}

// In geometry.rs (or as a helper function on Rect)
pub fn calculate_absolute_rect(
    metric: &LayoutMetrics,
    relative_rect: &Rect,
    screen_width: f64,
    screen_height: f64,
) -> Rect {
    // 1. Calculate Width (W) and Height (H)
    let (abs_w, abs_h) = match metric.size_mode {
        LengthMode::AbsolutePixels => (relative_rect.w, relative_rect.h),
        LengthMode::Percentage => (relative_rect.w * screen_width, relative_rect.h * screen_height),
    };

    // 2. Calculate Anchor Offset (AX, AY)
    let (anchor_x, anchor_y) = match metric.anchor {
        AnchorPoint::TopLeft => (0.0, 0.0),
        AnchorPoint::TopRight => (screen_width, 0.0),
        AnchorPoint::BottomLeft => (0.0, screen_height),
        AnchorPoint::BottomRight => (screen_width, screen_height),
        AnchorPoint::Center => (screen_width / 2.0, screen_height / 2.0),
    };

    // 3. Calculate Relative Position (RX, RY) - UPDATED
    let (rel_x, rel_y) = match metric.position_mode {
        LengthMode::AbsolutePixels => (relative_rect.x, relative_rect.y),
        LengthMode::Percentage => (relative_rect.x * screen_width, relative_rect.y * screen_height),
    };
    
    // 4. Final X and Y Position
    let final_x = anchor_x + rel_x - (match metric.anchor {
        // If anchored right, shift left by the component's width
        AnchorPoint::TopRight | AnchorPoint::BottomRight => abs_w,
        // If anchored center, shift left by half the component's width
        AnchorPoint::Center => abs_w / 2.0,
        // Otherwise (TopLeft, BottomLeft), no horizontal shift needed
        _ => 0.0,
    });
    
    let final_y = anchor_y + rel_y - (match metric.anchor {
        // If anchored bottom, shift up by the component's height
        AnchorPoint::BottomLeft | AnchorPoint::BottomRight => abs_h,
        // If anchored center, shift up by half the component's height
        AnchorPoint::Center => abs_h / 2.0,
        // Otherwise (TopLeft, TopRight), no vertical shift needed
        _ => 0.0,
    });

    Rect::new_rounded(final_x, final_y, abs_w, abs_h, relative_rect.radius)
}