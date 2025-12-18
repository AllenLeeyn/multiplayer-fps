pub mod components;
pub mod context;
pub mod driver;
pub mod events;
pub mod fonts;
pub mod geometry;
pub mod layout;
pub mod manager;

pub use components::Component;
pub use context::UIMainContext;
pub use driver::AppDriver;
pub use events::{ComponentUpdate, UIEvent};
pub use geometry::{IntRect, Rect};
pub use layout::{AnchorPoint, LayoutContext, LayoutMetrics, LengthMode, calculate_absolute_rect};
pub use manager::UIManager;

pub use winit::event::WindowEvent::KeyboardInput;
pub use winit::event::{
    ElementState, // Pressed or Released
    MouseButton,  // Left, Right, Middle, etc.
    // You can add more like VirtualKeyCode, DeviceEvent, etc., as needed
    WindowEvent, // The main event enum
};
pub use winit::keyboard::{KeyCode, PhysicalKey};

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

    // -------------------------------------------------
    // Core colors
    // -------------------------------------------------
    pub const BLACK: Color = Color::rgb(0, 0, 0);
    pub const WHITE: Color = Color::rgb(255, 255, 255);

    pub const RED: Color = Color::rgb(255, 0, 0);
    pub const GREEN: Color = Color::rgb(0, 255, 0);
    pub const BLUE: Color = Color::rgb(0, 0, 255);

    pub const CYAN: Color = Color::rgb(0, 255, 255);
    pub const MAGENTA: Color = Color::rgb(255, 0, 255);
    pub const YELLOW: Color = Color::rgb(255, 255, 0);

    // -------------------------------------------------
    // Grayscale ramp (VERY useful for UI)
    // -------------------------------------------------
    pub const GRAY_10: Color = Color::rgb(26, 26, 26);
    pub const GRAY_20: Color = Color::rgb(51, 51, 51);
    pub const GRAY_30: Color = Color::rgb(77, 77, 77);
    pub const GRAY_40: Color = Color::rgb(102, 102, 102);
    pub const GRAY_50: Color = Color::rgb(128, 128, 128);
    pub const GRAY_60: Color = Color::rgb(153, 153, 153);
    pub const GRAY_70: Color = Color::rgb(179, 179, 179);
    pub const GRAY_80: Color = Color::rgb(204, 204, 204);
    pub const GRAY_90: Color = Color::rgb(230, 230, 230);

    pub const LIGHT_GRAY: Color = Color::GRAY_80;
    pub const DARK_GRAY: Color = Color::GRAY_20;

    // -------------------------------------------------
    // Accent / highlight colors
    // -------------------------------------------------
    pub const ORANGE: Color = Color::rgb(255, 165, 0);
    pub const PINK: Color = Color::rgb(255, 105, 180);
    pub const PURPLE: Color = Color::rgb(128, 0, 128);
    pub const TEAL: Color = Color::rgb(0, 128, 128);
    pub const LIME: Color = Color::rgb(191, 255, 0);
}
