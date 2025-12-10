pub mod components;
pub mod context;
pub mod events;
pub mod fonts;
pub mod geometry;
pub mod driver;
pub mod layers;
pub mod layout;

pub use context::UIMainContext;
pub use geometry::{Rect, Bounds, calculate_absolute_rect, IntRect};
pub use driver::AppDriver;
pub use events::{ComponentUpdate, UIEvent};
pub use layout::{AnchorPoint, LayoutMetrics, LengthMode};
pub use components::Component;

pub use winit::event::{
    WindowEvent,    // The main event enum
    ElementState,   // Pressed or Released
    MouseButton,    // Left, Right, Middle, etc.
    // You can add more like VirtualKeyCode, DeviceEvent, etc., as needed
};
pub use winit::keyboard::{KeyCode, PhysicalKey};
pub use winit::event::WindowEvent::KeyboardInput;

/// Represents a simple RGB color with 8-bit channels.
/// Used universally across UI components and rendering logic (u8 format).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
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

    pub const BLACK: Color = Color::new(0, 0, 0, 255);
    pub const WHITE: Color = Color::new(255, 255, 255, 255);
    pub const RED: Color = Color::new(255, 0, 0, 255);
    pub const BLUE: Color = Color::new(0, 0, 255, 255);
    pub const MAGENTA: Color = Color::new(255, 0, 255, 255);
    pub const GREEN: Color = Color::new(0, 255, 0, 255);
    pub const DARK_GRAY: Color = Color::new(40, 40, 40, 255);
    pub const LIGHT_BLUE: Color = Color::new(150, 200, 255, 255);
}