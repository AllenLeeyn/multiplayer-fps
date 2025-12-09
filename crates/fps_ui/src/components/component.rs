use std::fmt::Debug;
use crate::geometry::Bounds;

// --- Abstract Types ---
use super::super::{
    UIMainContext,
    ComponentUpdate,
    UIEvent,
    UIInputEvent,
    LayoutMetrics
};

/// Defines the mandatory contract for all UI elements.
pub trait Component: Send + Sync + Debug {// Identification
    fn id(&self) -> &str;
    
    // Geometry
    fn bounds(&self) -> Bounds;
    
    // Layout (NEW: Defines how the bounds should be interpreted and anchored)
    fn layout_metrics(&self) -> LayoutMetrics;

    // Logic/Interaction (returns application events)
    fn handle_input(&mut self, event: &UIInputEvent) -> Vec<UIEvent>;
    
    // Data Synchronization (returns true if a visual change requires a redraw)
    fn apply_update(&mut self, update: &ComponentUpdate) -> bool;

    // Redraw Optimization (NEW: Signals if component visual state changed since last frame)
    fn requires_redraw(&mut self) -> bool;
    
    // Rendering
    fn draw(&self, frame: &mut [u8], context: &mut UIMainContext, screen_width: u32, screen_height: u32);
}
