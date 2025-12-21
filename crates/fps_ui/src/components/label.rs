use super::super::{
    Color, Component, ComponentUpdate, LayoutMetrics, Rect, UIEvent, UIMainContext, WindowEvent,
    calculate_absolute_rect,
};
use std::any::Any;

/// Defines the style and content of a text label.
#[derive(Debug, Clone)]
pub struct Label {
    id: String,
    text: String,
    font_size: f32,
    color: Color,
    bounds: Rect,
    metrics: LayoutMetrics,
    needs_redraw: bool,
}

impl Label {
    pub fn new(
        id: String,
        text: String,
        font_size: f32,
        color: Color,
        bounds: Rect,
        metrics: LayoutMetrics,
    ) -> Self {
        Label {
            id,
            text,
            font_size,
            color,
            bounds,
            metrics,
            needs_redraw: true,
        }
    }
}

impl Component for Label {
    fn id(&self) -> &str {
        &self.id
    }

    fn bounds(&self) -> Rect {
        self.bounds
    }

    fn layout_metrics(&self) -> LayoutMetrics {
        self.metrics
    }

    fn handle_input(
        &mut self,
        _event: &WindowEvent,
        _cursor_pos: Option<(f64, f64)>,
    ) -> Vec<UIEvent> {
        Vec::new()
    }

    fn apply_update(&mut self, update: &ComponentUpdate) -> bool {
        match update {
            ComponentUpdate::SetText(_, new_text) => {
                if self.text != *new_text {
                    self.text = new_text.clone();
                    self.needs_redraw = true;
                    return true;
                }
                false
            }
            ComponentUpdate::SetPosition(_, x, y) => {
                self.bounds.x = *x;
                self.bounds.y = *y;
                self.needs_redraw = true;
                true
            }
            _ => false,
        }
    }

    fn requires_redraw(&mut self) -> bool {
        if self.needs_redraw {
            self.needs_redraw = false;
            true
        } else {
            false
        }
    }

    fn draw(&self, frame: &mut [u8], context: &mut UIMainContext) {
        let absolute_rect = calculate_absolute_rect(&self.metrics, &self.bounds, &context.layout);

        let start_x = absolute_rect.x;
        let start_y = absolute_rect.y;
        let (logical_width, logical_height) = context.layout.size_as_u32();

        context.font_manager.draw_text(
            frame,
            &self.text,
            self.font_size,
            self.color,
            start_x,
            start_y,
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
