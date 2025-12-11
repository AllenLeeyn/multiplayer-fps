use super::super::{
    Bounds, Color, Component, ComponentUpdate, LayoutMetrics, UIEvent, UIMainContext, WindowEvent,
    calculate_absolute_rect,
};

/// A specialized component that displays the current Frames Per Second (FPS).
#[derive(Debug, Clone)]
pub struct FpsComponent {
    pub id: String,
    pub bounds: Bounds,
    pub layout_metrics: LayoutMetrics,
    pub font_size: f32,
    pub color: Color,

    // FpsComponent specific fields
    current_fps: u32,
    text: String,
    needs_redraw: bool,
}

impl FpsComponent {
    pub fn new(
        id: String,
        font_size: f32,
        color: Color,
        relative_bounds: Bounds,
        metrics: LayoutMetrics,
    ) -> Self {
        let initial_fps = 0;
        let initial_text = format!("FPS: {}", initial_fps);

        Self {
            id,
            bounds: relative_bounds,
            layout_metrics: metrics,
            font_size,
            color,
            current_fps: initial_fps,
            text: initial_text,
            needs_redraw: true,
        }
    }

    pub fn update_fps(&mut self, new_fps: u32) -> bool {
        if self.current_fps != new_fps {
            self.current_fps = new_fps;
            self.text = format!("FPS: {}", new_fps);
            self.needs_redraw = true;
            true
        } else {
            false
        }
    }
}

impl Component for FpsComponent {
    fn id(&self) -> &str {
        &self.id
    }

    fn bounds(&self) -> Bounds {
        self.bounds
    }

    fn layout_metrics(&self) -> LayoutMetrics {
        self.layout_metrics
    }

    fn handle_input(&mut self, _event: &WindowEvent) -> Vec<UIEvent> {
        Vec::new()
    }

    /// Handles incoming updates from the application layer.
    fn apply_update(&mut self, update: &ComponentUpdate) -> bool {
        match update {
            ComponentUpdate::SetValue(id, new_value) if id == self.id() => {
                let new_fps = new_value.round() as u32;
                self.update_fps(new_fps)
            }
            _ => false,
        }
    }

    fn requires_redraw(&mut self) -> bool {
        std::mem::take(&mut self.needs_redraw)
    }

    fn draw(
        &self,
        frame: &mut [u8],
        context: &mut UIMainContext,
        screen_width: u32,
        screen_height: u32,
    ) {
        let screen_w_f64 = screen_width as f64;
        let screen_h_f64 = screen_height as f64;

        let absolute_rect = calculate_absolute_rect(
            &self.layout_metrics,
            &self.bounds,
            screen_w_f64,
            screen_h_f64,
        );

        context.font_manager.draw_text(
            frame,
            &self.text,
            self.font_size,
            self.color,
            absolute_rect.x,
            absolute_rect.y,
            context.global_scale_factor,
            screen_width,
            screen_height,
        );
    }

    fn get_text(&self) -> &str {
        &self.text
    }
}
