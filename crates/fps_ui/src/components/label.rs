use super::super::{
    Bounds, Color, Component, ComponentUpdate, LayoutMetrics, UIEvent, UIMainContext, WindowEvent,
    calculate_absolute_rect,
};

/// Defines the style and content of a text label.
#[derive(Debug, Clone)]
pub struct Label {
    id: String,
    text: String,
    font_size: f32,
    color: Color,
    bounds: Bounds,
    metrics: LayoutMetrics,
    needs_redraw: bool,
}

impl Label {
    pub fn new(
        id: String,
        text: String,
        font_size: f32,
        color: Color,
        bounds: Bounds,
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

    fn bounds(&self) -> Bounds {
        self.bounds
    }

    fn layout_metrics(&self) -> LayoutMetrics {
        self.metrics
    }

    fn handle_input(&mut self, _event: &WindowEvent) -> Vec<UIEvent> {
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

    fn draw(
        &self,
        frame: &mut [u8],
        context: &mut UIMainContext,
        screen_width: u32,
        screen_height: u32,
    ) {
        // --- 1. Calculate Absolute Position and Size ---
        let screen_w_f64 = screen_width as f64;
        let screen_h_f64 = screen_height as f64;

        let absolute_rect = calculate_absolute_rect(
            &self.metrics,
            &self.bounds,
            screen_w_f64,
            screen_h_f64,
        );

        let start_x = absolute_rect.x;
        let start_y = absolute_rect.y;

        // --- 2. Call the FontManager for Drawing ---
        context.font_manager.draw_text(
            frame,
            &self.text,
            self.font_size,
            self.color,
            start_x,
            start_y,
            context.global_scale_factor,
            screen_width,
            screen_height,
        );
    }

    fn get_text(&self) -> &str {
        &self.text
    }
}
