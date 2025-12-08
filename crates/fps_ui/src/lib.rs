pub mod components;
pub mod context;
pub mod events;
pub mod font;
pub mod geometry;
pub mod driver;
pub mod layers;

pub use context::UIMainContext;
pub use geometry::Rect;
pub use driver::AppDriver;
pub use events::{ComponentUpdate, UIEvent, UIInputEvent};

/// Represents a simple RGB color with 8-bit channels.
/// Used universally across UI components and rendering logic (u8 format).
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Color {
    pub r: u8,
    pub g: u8,
    pub b: u8,
    pub a: u8,
}
impl Color {
    /// Creates a new Color instance.
    pub const fn new(r: u8, g: u8, b: u8, a: u8) -> Self {
        Color { r, g, b, a }
    }

    /// Creates a fully opaque Color instance from RGB values (Alpha = 255).
    pub const fn rgb(r: u8, g: u8, b: u8) -> Self {
        // Renamed from 'new' to 'rgb' for clarity.
        Color { r, g, b, a: 255 }
    }

    /// Predefined solid color: Black (Opaque)
    pub const BLACK: Color = Color::new(0, 0, 0, 255);
    /// Predefined solid color: White (Opaque)
    pub const WHITE: Color = Color::new(255, 255, 255, 255);
    /// Predefined solid color: Red (Opaque)
    pub const RED: Color = Color::new(255, 0, 0, 255);
    /// Predefined solid color: Blue (Opaque)
    pub const BLUE: Color = Color::new(0, 0, 255, 255);
    /// Predefined solid color: Debug Magenta (Opaque)
    pub const MAGENTA: Color = Color::new(255, 0, 255, 255);
    /// Predefined solid color: Green (Opaque)
    pub const GREEN: Color = Color::new(0, 255, 0, 255);
}