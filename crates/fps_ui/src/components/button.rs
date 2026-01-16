//! # Button Component
//!
//! An interactive button component that responds to mouse clicks and hover states.
//! Displays text centered both horizontally and vertically.

use super::super::{
    Color, Component, ComponentUpdate, ElementState, IntRect, LayoutMetrics, MouseButton, Rect,
    UIEvent, UIMainContext, WindowEvent, calculate_absolute_rect, draw_filled_box
};
use ab_glyph::{Font, ScaleFont};
use std::any::Any;

/// An interactive button component.
///
/// Buttons respond to mouse clicks and hover states, emitting `ButtonClicked` events
/// when clicked. The button's background color changes on hover to provide visual feedback.
///
/// # Example
///
/// ```rust,no_run
/// use fps_ui::components::Button;
/// use fps_ui::{Color, Rect, layout::LayoutMetrics};
///
/// let button = Button::new(
///     "start_button",
///     "Start Game",
///     Rect::new(100.0, 100.0, 200.0, 50.0),
///     LayoutMetrics::default(),
///     Color::WHITE,  // text color
///     Color::BLUE,   // background color
///     Color::CYAN,   // hover color
/// );
/// ```
#[derive(Debug)]
pub struct Button {
    id: String,
    bounds: Rect,
    layout: LayoutMetrics,

    // Visual State
    text: String,
    font_size: f32,
    background_color: Color,
    hover_color: Color,
    text_color: Color,

    // Interaction State
    is_hovered: bool,
    is_pressed: bool,
    is_visible: bool,
    redraw_required: bool,
}

impl Button {
    /// Creates a new button component.
    ///
    /// # Arguments
    ///
    /// * `id` - Unique identifier for the button
    /// * `text` - Text to display on the button
    /// * `bounds` - Button's bounding rectangle (relative coordinates)
    /// * `layout` - Layout metrics for positioning
    /// * `text_color` - Color of the button text
    /// * `background_color` - Normal background color
    /// * `hover_color` - Background color when hovered
    ///
    /// # Returns
    ///
    /// A new `Button` instance with default font size of 24 pixels.
    pub fn new(
        id: &str,
        text: &str,
        bounds: Rect,
        layout: LayoutMetrics,
        text_color: Color,
        background_color: Color,
        hover_color: Color,
    ) -> Self {
        Button {
            id: id.to_string(),
            bounds,
            layout,
            text: text.to_string(),
            font_size: 24.0,
            background_color,
            hover_color,
            text_color,
            is_hovered: false,
            is_pressed: false,
            is_visible: true,
            redraw_required: true,
        }
    }

    /// Gets the current background color based on interaction state.
    ///
    /// Returns `hover_color` if the button is hovered, otherwise `background_color`.
    fn current_bg_color(&self) -> Color {
        if self.is_hovered {
            self.hover_color
        } else {
            self.background_color
        }
    }
}

impl Component for Button {
    fn id(&self) -> &str {
        &self.id
    }

    fn bounds(&self) -> Rect {
        self.bounds
    }

    fn layout_metrics(&self) -> LayoutMetrics {
        self.layout
    }

    fn handle_input(
        &mut self,
        event: &WindowEvent,
        _cursor_pos: Option<(f64, f64)>,
    ) -> Vec<UIEvent> {
        let mut events = Vec::new();

        match event {
            WindowEvent::MouseInput { state, button, .. } => {
                if *button == MouseButton::Left {
                    let old_pressed = self.is_pressed;

                    // The button must be hovered to be pressed
                    self.is_pressed = *state == ElementState::Pressed && self.is_hovered;

                    // If released and was pressed, and is still hovered, generate a Click event
                    if old_pressed && *state == ElementState::Released && self.is_hovered {
                        events.push(UIEvent::ButtonClicked(self.id.clone()));
                    }

                    if old_pressed != self.is_pressed {
                        self.redraw_required = true;
                    }
                }
            }
            _ => {}
        }

        events
    }

    fn apply_update(&mut self, update: &ComponentUpdate) -> bool {
        match update {
            ComponentUpdate::SetVisibility(_, visibility) => {
                self.is_visible = *visibility;
                true
            }
            ComponentUpdate::SetText(_, new_text) => {
                self.text = new_text.clone();
                self.redraw_required = true;
                true
            }
            ComponentUpdate::SetHovered(_id, is_hovered) => {
                if self.is_hovered != *is_hovered {
                    self.is_hovered = *is_hovered;
                    self.redraw_required = true;
                    true
                } else {
                    false
                }
            }
            _ => false,
        }
    }

    fn requires_redraw(&mut self) -> bool {
        let required = self.redraw_required;
        self.redraw_required = false;
        required
    }

    fn draw(&self, frame: &mut [u8], context: &mut UIMainContext) {
        if !self.is_visible {
            return;
        }

        let bounds_f64 = calculate_absolute_rect(&self.layout, &self.bounds, &context.layout);
        let (logical_width, logical_height) = context.layout.size_as_u32();

        let abs_rect: IntRect = bounds_f64.to_int_rect();
        let bg_color = self.current_bg_color();
        let text_color = self.text_color;

        // 1. Draw Background
        draw_filled_box(
            frame,
            logical_width,
            logical_height,
            abs_rect.x as u32,
            abs_rect.y as u32,
            abs_rect.w,
            abs_rect.h,
            bg_color,
        );

        // 2. Draw Text (Vertically and Horizontally Centered)
        let text_to_draw = self.text.as_str();

        let (scaled_ascent, scaled_descent) = {
            let scale = context.font_manager.calculate_scale(self.font_size);
            let font_arc = context.font_manager.get_primary_font();
            let scaled_font = font_arc.as_scaled(scale);

            let ascent = (scaled_font.ascent()) as f64;
            let descent = (scaled_font.descent()) as f64;
            (ascent, descent)
        };

        // Vertical Centering: Center of box - (offset from baseline to font center)
        let font_center_offset = (scaled_ascent + scaled_descent) / 2.0;
        let component_center_y = bounds_f64.y + bounds_f64.h / 2.0;
        let text_baseline_y = component_center_y - font_center_offset;

        // Horizontal Centering:
        let text_width = context
            .font_manager
            .measure_text_width(text_to_draw, self.font_size);
        let text_start_x = bounds_f64.x + (bounds_f64.w / 2.0) - (text_width / 2.0);

        context.font_manager.draw_text(
            frame,
            text_to_draw,
            self.font_size,
            text_color,
            text_start_x,
            text_baseline_y,
            logical_width,
            logical_height,
        );
    }

    fn get_text(&self) -> &str {
        &self.text
    }

    fn as_any(&self) -> &dyn Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }
}
