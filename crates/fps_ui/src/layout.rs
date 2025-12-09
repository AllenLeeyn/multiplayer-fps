
/// Defines the corner of the window the component's position is relative to.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum AnchorPoint {
    TopLeft,        // Default: (0, 0) is top-left
    TopRight,       // (Screen_W, 0)
    BottomLeft,     // (0, Screen_H)
    BottomRight,    // (Screen_W, Screen_H)
    Center,         // (Screen_W / 2, Screen_H / 2)
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum LengthMode {
    AbsolutePixels, // Value is interpreted as fixed pixels (f64).
    Percentage,     // Value is interpreted as a percentage (0.0 to 1.0) of the screen dimension.
}

#[derive(Debug, Clone, Copy)]
pub struct LayoutMetrics {
    pub anchor: AnchorPoint,
    pub position_mode: LengthMode,
    pub size_mode: LengthMode,
}

impl Default for LayoutMetrics {
    fn default() -> Self {
        LayoutMetrics {
            anchor: AnchorPoint::TopLeft,
            position_mode: LengthMode::AbsolutePixels,
            size_mode: LengthMode::AbsolutePixels,
        }
    }
}
