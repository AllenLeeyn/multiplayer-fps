use std::any::Any;
use std::fmt::Debug;

// --- Abstract Types ---
use super::super::{ComponentUpdate, LayoutMetrics, Rect, UIEvent, UIMainContext, WindowEvent};

/// Defines the mandatory contract for all UI elements.
pub trait Component: Send + Sync + Debug {
    // Identification
    fn id(&self) -> &str;

    // Compoenent location and size parameter
    fn bounds(&self) -> Rect;

    // Compoenent location and size parameter meterics
    fn layout_metrics(&self) -> LayoutMetrics;

    // Logic/Interaction (returns application events)
    fn handle_input(&mut self, event: &WindowEvent, cursor_pos: Option<(f64, f64)>)
    -> Vec<UIEvent>;

    // Data Synchronization (returns true if a visual change requires a redraw)
    fn apply_update(&mut self, update: &ComponentUpdate) -> bool;

    // Redraw Optimization (intends to notify when to redraw)
    fn requires_redraw(&mut self) -> bool;

    // set component is focus or not
    fn set_focus(&mut self, is_focused: bool) -> bool {
        self.apply_update(&ComponentUpdate::SetFocus(
            self.id().to_string(),
            is_focused,
        ))
    }

    // set component is hovered or not
    fn set_hovered(&mut self, is_hovered: bool) -> bool {
        self.apply_update(&ComponentUpdate::SetHovered(
            self.id().to_string(),
            is_hovered,
        ))
    }

    // Rendering
    fn draw(&self, frame: &mut [u8], context: &mut UIMainContext);

    fn get_text(&self) -> &str;

    fn update(&mut self) {}

    fn as_any(&self) -> &dyn Any;
    fn as_any_mut(&mut self) -> &mut dyn Any;
}
