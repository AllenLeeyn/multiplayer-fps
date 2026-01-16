//! # Layout Module
//!
//! Provides a flexible layout system for positioning and sizing UI components.
//! Supports both pixel-based and percentage-based positioning with multiple anchor points.

use super::Rect;

/// Specifies how length values are interpreted.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum LengthMode {
    /// Values are interpreted as logical pixels.
    Px,
    
    /// Values are interpreted as percentages (0.0 to 1.0).
    Percent,
}

/// Anchor point for component positioning.
///
/// Determines which corner or center point of the component is used as the
/// reference for positioning relative to the anchor point.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum AnchorPoint {
    /// Anchor at the top-left corner.
    TopLeft,
    
    /// Anchor at the top-right corner.
    TopRight,
    
    /// Anchor at the bottom-left corner.
    BottomLeft,
    
    /// Anchor at the bottom-right corner.
    BottomRight,
    
    /// Anchor at the center point.
    Center,
}

/// Layout metrics defining how a component is positioned and sized.
///
/// Combines anchor point, positioning mode, and sizing mode to create
/// flexible layout behavior.
#[derive(Debug, Clone, Copy)]
pub struct LayoutMetrics {
    pub anchor: AnchorPoint,
    pub positioning: LengthMode,
    pub sizing: LengthMode,
}

impl Default for LayoutMetrics {
    fn default() -> Self {
        LayoutMetrics {
            anchor: AnchorPoint::TopLeft,
            positioning: LengthMode::Px,
            sizing: LengthMode::Px,
        }
    }
}

/// Layout context containing the logical canvas dimensions.
///
/// Used by the layout system to resolve percentage-based positions and sizes
/// and to calculate anchor point positions.
#[derive(Debug, Clone, Copy)]
pub struct LayoutContext {
    pub logical_width: f64,
    pub logical_height: f64,
}

impl LayoutContext {
    pub fn new(logical_width: f64, logical_height: f64) -> Self {
        Self {
            logical_width,
            logical_height,
        }
    }

    /// Resolves a relative rectangle to absolute coordinates.
    ///
    /// Applies anchor point, positioning mode, and sizing mode to convert
    /// a component's local rectangle to screen coordinates.
    ///
    /// # Arguments
    ///
    /// * `rect` - The relative rectangle (in component-local coordinates)
    /// * `metrics` - The layout metrics defining positioning and sizing
    ///
    /// # Returns
    ///
    /// An absolute rectangle in logical screen coordinates.
    pub fn resolve_rect(&self, rect: &Rect, metrics: &LayoutMetrics) -> Rect {
        let (abs_w, abs_h) = match metrics.sizing {
            LengthMode::Px => (rect.w, rect.h),
            LengthMode::Percent => (rect.w * self.logical_width, rect.h * self.logical_height),
        };

        let (anchor_x, anchor_y) = match metrics.anchor {
            AnchorPoint::TopLeft => (0.0, 0.0),
            AnchorPoint::TopRight => (self.logical_width, 0.0),
            AnchorPoint::BottomLeft => (0.0, self.logical_height),
            AnchorPoint::BottomRight => (self.logical_width, self.logical_height),
            AnchorPoint::Center => (self.logical_width / 2.0, self.logical_height / 2.0),
        };

        let (rel_x, rel_y) = match metrics.positioning {
            LengthMode::Px => (rect.x, rect.y),
            LengthMode::Percent => (rect.x * self.logical_width, rect.y * self.logical_height),
        };

        let final_x = anchor_x + rel_x
            - match metrics.anchor {
                AnchorPoint::TopRight | AnchorPoint::BottomRight => abs_w,
                AnchorPoint::Center => abs_w / 2.0,
                _ => 0.0,
            };

        let final_y = anchor_y + rel_y
            - match metrics.anchor {
                AnchorPoint::BottomLeft | AnchorPoint::BottomRight => abs_h,
                AnchorPoint::Center => abs_h / 2.0,
                _ => 0.0,
            };

        Rect::new(final_x, final_y, abs_w, abs_h)
    }

    pub fn size(&self) -> (f64, f64) {
        (self.logical_width, self.logical_height)
    }

    pub fn size_as_u32(&self) -> (u32, u32) {
        (self.logical_width as u32, self.logical_height as u32)
    }

    pub fn size_as_usize(&self) -> (usize, usize) {
        (self.logical_width as usize, self.logical_height as usize)
    }
}

/// Calculates the absolute position and size of a UI component.
///
/// Converts a component's relative rectangle (defined in its local coordinate space)
/// to absolute logical coordinates based on layout metrics and context.
///
/// The returned rectangle is in logical space and will be scaled to physical
/// pixels by the `AppDriver` during rendering.
///
/// # Arguments
///
/// * `metrics` - Layout metrics defining anchor, positioning, and sizing
/// * `relative_rect` - Component's local rectangle
/// * `layout` - Layout context with canvas dimensions
///
/// # Returns
///
/// An absolute rectangle in logical coordinates.
pub fn calculate_absolute_rect(
    metrics: &LayoutMetrics,
    relative_rect: &Rect,
    layout: &LayoutContext,
) -> Rect {
    // Determine width and height
    let width = match metrics.sizing {
        LengthMode::Px => relative_rect.w,
        LengthMode::Percent => relative_rect.w * layout.logical_width,
    };
    let height = match metrics.sizing {
        LengthMode::Px => relative_rect.h,
        LengthMode::Percent => relative_rect.h * layout.logical_height,
    };

    // Determine anchor offset
    let (anchor_x, anchor_y) = match metrics.anchor {
        AnchorPoint::TopLeft => (0.0, 0.0),
        AnchorPoint::TopRight => (layout.logical_width, 0.0),
        AnchorPoint::BottomLeft => (0.0, layout.logical_height),
        AnchorPoint::BottomRight => (layout.logical_width, layout.logical_height),
        AnchorPoint::Center => (layout.logical_width / 2.0, layout.logical_height / 2.0),
    };

    // Relative position offset
    let (rel_x, rel_y) = match metrics.positioning {
        LengthMode::Px => (relative_rect.x, relative_rect.y),
        LengthMode::Percent => (
            relative_rect.x * layout.logical_width,
            relative_rect.y * layout.logical_height,
        ),
    };

    // Apply anchor shifts
    let final_x = anchor_x + rel_x
        - match metrics.anchor {
            AnchorPoint::TopRight | AnchorPoint::BottomRight => width,
            AnchorPoint::Center => width / 2.0,
            _ => 0.0,
        };

    let final_y = anchor_y + rel_y
        - match metrics.anchor {
            AnchorPoint::BottomLeft | AnchorPoint::BottomRight => height,
            AnchorPoint::Center => height / 2.0,
            _ => 0.0,
        };

    Rect::new(final_x, final_y, width, height)
}
