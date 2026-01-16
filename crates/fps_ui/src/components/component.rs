//! # Component Trait
//!
//! Defines the core trait that all UI components must implement.
//! This trait provides a unified interface for rendering, input handling, and state management.

use std::any::Any;
use std::fmt::Debug;

use super::super::{ComponentUpdate, LayoutMetrics, Rect, UIEvent, UIMainContext, WindowEvent};

/// Defines the mandatory contract for all UI elements.
///
/// All UI components must implement this trait to participate in the UI system.
/// Components handle their own rendering, input processing, and state updates.
pub trait Component: Send + Sync + Debug {
    /// Returns the unique identifier for this component.
    fn id(&self) -> &str;

    /// Returns the component's bounding rectangle in local coordinates.
    fn bounds(&self) -> Rect;

    /// Returns the layout metrics for positioning and sizing.
    fn layout_metrics(&self) -> LayoutMetrics;

    /// Handles input events and returns UI events for the application to process.
    ///
    /// # Arguments
    ///
    /// * `event` - The window event to process
    /// * `cursor_pos` - Current cursor position in logical coordinates, if available
    ///
    /// # Returns
    ///
    /// A vector of UI events that the application should handle (e.g., ButtonClicked).
    fn handle_input(&mut self, event: &WindowEvent, cursor_pos: Option<(f64, f64)>) -> Vec<UIEvent>;

    /// Applies an update to the component's state.
    ///
    /// # Arguments
    ///
    /// * `update` - The component update to apply
    ///
    /// # Returns
    ///
    /// `true` if the update requires a redraw, `false` otherwise.
    fn apply_update(&mut self, update: &ComponentUpdate) -> bool;

    /// Checks if the component requires a redraw.
    ///
    /// Used for optimization to avoid unnecessary rendering.
    fn requires_redraw(&mut self) -> bool;

    /// Sets the focus state of the component.
    ///
    /// # Arguments
    ///
    /// * `is_focused` - Whether the component should be focused
    ///
    /// # Returns
    ///
    /// `true` if the focus change requires a redraw.
    fn set_focus(&mut self, is_focused: bool) -> bool {
        self.apply_update(&ComponentUpdate::SetFocus(
            self.id().to_string(),
            is_focused,
        ))
    }

    /// Sets the hovered state of the component.
    ///
    /// # Arguments
    ///
    /// * `is_hovered` - Whether the component should be hovered
    ///
    /// # Returns
    ///
    /// `true` if the hover change requires a redraw.
    fn set_hovered(&mut self, is_hovered: bool) -> bool {
        self.apply_update(&ComponentUpdate::SetHovered(
            self.id().to_string(),
            is_hovered,
        ))
    }

    /// Renders the component to the frame buffer.
    ///
    /// # Arguments
    ///
    /// * `frame` - The pixel frame buffer to draw into (RGBA format)
    /// * `context` - The UI context containing fonts, textures, and layout information
    fn draw(&self, frame: &mut [u8], context: &mut UIMainContext);

    /// Returns the text content of the component, if applicable.
    ///
    /// For non-text components, this may return an empty string.
    fn get_text(&self) -> &str;

    /// Updates the component's internal state (called each frame).
    ///
    /// Override this for components that need per-frame updates (e.g., FPS counter).
    fn update(&mut self) {}

    /// Returns a type-erased reference for downcasting.
    fn as_any(&self) -> &dyn Any;

    /// Returns a type-erased mutable reference for downcasting.
    fn as_any_mut(&mut self) -> &mut dyn Any;
}
