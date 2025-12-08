use crate::context::UIMainContext;
use crate::events::{ComponentUpdate, UIEvent};
use winit::event::WindowEvent;
use winit::window::Window;

/// Defines the mandatory contract for all UI elements.
pub trait Component: Send + Sync {
    fn id(&self) -> &str;
    fn handle_input(&mut self, event: &WindowEvent, window: &Window) -> Vec<UIEvent>;
    fn draw(&self, frame: &mut [u8], context: &UIMainContext);
    fn apply_update(&mut self, update: &ComponentUpdate);
}

/// Simple struct for defining a rectangular boundary in screen coordinates.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Bounds {
    pub x: u32,
    pub y: u32,
    pub width: u32,
    pub height: u32,
}

impl Bounds {
    /// Checks if a given coordinate (e.g., mouse position) falls within these bounds.
    pub fn contains(&self, px: f64, py: f64) -> bool {
        px >= self.x as f64
            && py >= self.y as f64
            && px < (self.x + self.width) as f64
            && py < (self.y + self.height) as f64
    }
}
