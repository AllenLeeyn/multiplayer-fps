use std::fmt::Debug;
use crate::geometry::Bounds;

// --- Abstract Types ---
use super::super::{UIMainContext, ComponentUpdate, UIEvent, UIInputEvent}; // Use abstract input

/// Defines the mandatory contract for all UI elements.
pub trait Component: Send + Sync + Debug {
    fn id(&self) -> &str;
    fn bounds(&self) -> Bounds;
    fn handle_input(&mut self, event: &UIInputEvent) -> Vec<UIEvent>;
    fn draw(&self, frame: &mut [u8], context: &UIMainContext, screen_width: u32, screen_height: u32);
    fn apply_update(&mut self, update: &ComponentUpdate) -> bool;
}
