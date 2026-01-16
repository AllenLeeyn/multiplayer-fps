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

use glam::U8Vec4;
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
    pub value: U8Vec4,
}

impl Color {
    /// Creates a new Color instance.
    pub const fn new(r: u8, g: u8, b: u8, a: u8) -> Self {
        Self {
            value: U8Vec4::new(r, g, b, a),
        }
    }

    /// Creates a fully opaque Color instance from RGB values (Alpha = 255).
    pub const fn rgb(r: u8, g: u8, b: u8) -> Self {
        Self::new(r, g, b, 255)
    }

    pub fn set_alpha_f32(&mut self, alpha: f32) {
        let clamped = alpha.clamp(0.0, 1.0);
        self.value.w = (clamped * 255.0).round() as u8;
    }

    pub fn darken(&self, factor: f32) -> Color {
        let f = factor.clamp(0.0, 1.0);

        Color {
            value: U8Vec4::new(
                ((self.value.x as f32) * f).round().min(255.0) as u8,
                ((self.value.y as f32) * f).round().min(255.0) as u8,
                ((self.value.z as f32) * f).round().min(255.0) as u8,
                self.value.w, // alpha unchanged
            ),
        }
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

#[inline]
fn draw_point(frame: &mut [u8], width: u32, height: u32, x: u32, y: u32, color: Color) {
    if x >= width || y >= height { return; }

    let offset = (y as usize * width as usize + x as usize) * 4;
    if offset + 3 >= frame.len() { return; }

    color.value.write_to_slice(&mut frame[offset..offset + 4]);
}

fn draw_triangle(
    frame: &mut [u8],
    width: u32,
    height: u32,
    cx: f32,
    cy: f32,
    angle: f32,
    size: f32,
    color: Color,
) {
    let forward = (angle.cos() * size, angle.sin() * size);
    let side = (
        (angle + std::f32::consts::FRAC_PI_2).cos() * size * 0.8,
        (angle + std::f32::consts::FRAC_PI_2).sin() * size * 0.8,
    );

    let p0 = (cx + forward.0, cy + forward.1);
    let p1 = (cx - forward.0 + side.0, cy - forward.1 + side.1);
    let p2 = (cx - forward.0 - side.0, cy - forward.1 - side.1);

    draw_filled_triangle(frame, width, height, p0, p1, p2, color);
}

fn draw_filled_triangle(
    frame: &mut [u8],
    width: u32,
    height: u32,
    p0: (f32, f32),
    p1: (f32, f32),
    p2: (f32, f32),
    color: Color,
) {
    // Sort by y-coordinate ascending (p0.y <= p1.y <= p2.y)
    let mut pts = [p0, p1, p2];
    pts.sort_by(|a, b| a.1.partial_cmp(&b.1).unwrap());

    let (p0, p1, p2) = (pts[0], pts[1], pts[2]);

    // Helper to interpolate x along a line
    fn interp_y(y0: f32, x0: f32, y1: f32, x1: f32, y: f32) -> f32 {
        if (y1 - y0).abs() < f32::EPSILON {
            x0
        } else {
            x0 + (x1 - x0) * (y - y0) / (y1 - y0)
        }
    }

    let y_start = p0.1.ceil() as u32;
    let y_end = p2.1.ceil() as u32;

    for y in y_start..y_end {
        let fy = y as f32;

        let xa = if fy < p1.1 {
            interp_y(p0.1, p0.0, p1.1, p1.0, fy)
        } else {
            interp_y(p1.1, p1.0, p2.1, p2.0, fy)
        };

        let xb = interp_y(p0.1, p0.0, p2.1, p2.0, fy);

        let x_start = xa.min(xb).ceil() as u32;
        let x_end = xa.max(xb).ceil() as u32;

        for x in x_start..x_end {
            draw_point(frame, width, height, x, y, color);
        }
    }
}

#[inline]
fn draw_filled_box(
    frame: &mut [u8],
    width: u32,
    height: u32,
    x: u32,
    y: u32,
    w: u32,
    h: u32,
    color: Color,
) {
    if w <= 0 || h <= 0 {
        return;
    }

    // Clip rectangle to frame bounds
    let x_start = x.min(width);
    let y_start = y.min(height);
    let x_end = (x + w).min(width);
    let y_end = (y + h).min(height);

    let row_width = x_end - x_start;
    if row_width == 0 || y_end <= y_start {
        return;
    }

    // Temporary buffer for a single row (row-blit)
    let mut row = vec![0u8; (row_width * 4) as usize];
    for i in 0..row_width as usize {
        let idx = i * 4;
        color.value.write_to_slice(&mut row[idx..idx + 4]);
    }

    // Copy the row into each row of the frame
    for yy in y_start..y_end {
        let offset = (yy * width + x_start) * 4;
        frame[offset as usize..offset as usize + row.len()].copy_from_slice(&row);
    }
}

#[inline]
pub fn draw_filled_bordered_box(
    frame: &mut [u8],
    width: u32,
    height: u32,
    x: u32,
    y: u32,
    w: u32,
    h: u32,
    fill_color: Color,
    border_color: Color,
) {
    if w == 0 || h == 0 {
        return;
    }

    // --- Draw interior ---
    if w > 1 && h > 1 {
        draw_filled_box(frame, width, height, x, y, w - 1, h - 1, fill_color);
    }

    // --- Draw right border ---
    draw_filled_box(frame, width, height, x + w - 1, y, 1, h, border_color);

    // --- Draw bottom border ---
    draw_filled_box(frame, width, height, x, y + h - 1, w, 1, border_color);
}
