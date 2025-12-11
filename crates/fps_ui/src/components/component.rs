use std::fmt::Debug;

// --- Abstract Types ---
use super::super::{
    Bounds,
    UIMainContext,
    ComponentUpdate,
    UIEvent,
    LayoutMetrics,
    WindowEvent
};

/// Defines the mandatory contract for all UI elements.
pub trait Component: Send + Sync + Debug {
    // Identification
    fn id(&self) -> &str;
    
    // Geometry
    fn bounds(&self) -> Bounds;
    
    // Layout (NEW: Defines how the bounds should be interpreted and anchored)
    fn layout_metrics(&self) -> LayoutMetrics;

    // Logic/Interaction (returns application events)
    fn handle_input(&mut self, event: &WindowEvent) -> Vec<UIEvent>;
    
    // Data Synchronization (returns true if a visual change requires a redraw)
    fn apply_update(&mut self, update: &ComponentUpdate) -> bool;

    // Redraw Optimization (NEW: Signals if component visual state changed since last frame)
    fn requires_redraw(&mut self) -> bool;
    
    // Focus Management (NEW: Consistent way to manage focus state)
    fn set_focus(&mut self, is_focused: bool) -> bool {
        // Default implementation uses apply_update for state change and redraw signaling
        self.apply_update(&ComponentUpdate::SetFocus(self.id().to_string(), is_focused))
    }

    fn set_hovered(&mut self, is_hovered: bool) -> bool {
        // Default implementation uses apply_update for state change and redraw signaling
        self.apply_update(&ComponentUpdate::SetHovered(self.id().to_string(), is_hovered))
    }

    // Rendering
    fn draw(&self, frame: &mut [u8], context: &mut UIMainContext, screen_width: u32, screen_height: u32);

    fn get_text(&self) -> &str;
}
