use std::any::Any;
use std::time::{Duration, Instant};

use super::super::{
    Color, Component, ComponentUpdate, LayoutMetrics, Rect, UIEvent, UIMainContext, WindowEvent,
    calculate_absolute_rect,
};

/// A specialized component that displays the current Frames Per Second (FPS).
#[derive(Debug, Clone)]
pub struct FpsComponent {
    pub id: String,
    pub bounds: Rect,
    pub layout_metrics: LayoutMetrics,
    pub font_size: f32,
    pub color: Color,

    // FpsComponent specific fields
    current_fps: u32,
    text: String,
    needs_redraw: bool,
    frame_count: u32,
    last_update: Option<Instant>,
}

impl FpsComponent {
    pub fn new(
        id: String,
        font_size: f32,
        color: Color,
        relative_bounds: Rect,
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
            frame_count: 0,
            last_update: None,
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

    fn bounds(&self) -> Rect {
        self.bounds
    }

    fn layout_metrics(&self) -> LayoutMetrics {
        self.layout_metrics
    }

    fn handle_input(
        &mut self,
        _event: &WindowEvent,
        _cursor_pos: Option<(f64, f64)>,
    ) -> Vec<UIEvent> {
        Vec::new()
    }

    /// Handles incoming updates from the application layer.
    fn apply_update(&mut self, _update: &ComponentUpdate) -> bool {
        false
    }

    fn requires_redraw(&mut self) -> bool {
        std::mem::take(&mut self.needs_redraw)
    }

    fn draw(&self, frame: &mut [u8], context: &mut UIMainContext) {
        let absolute_rect =
            calculate_absolute_rect(&self.layout_metrics, &self.bounds, &context.layout);
        let (logical_width, logical_height) = context.layout.size_as_u32();

        context.font_manager.draw_text(
            frame,
            &self.text,
            self.font_size,
            self.color,
            absolute_rect.x,
            absolute_rect.y,
            logical_width,
            logical_height,
        );
    }

    fn get_text(&self) -> &str {
        &self.text
    }

    fn update(&mut self) {
        let now = Instant::now();

        if self.last_update.is_none() {
            self.last_update = Some(now);
            self.frame_count = 0;
        }

        self.frame_count += 1;

        let last = self.last_update.unwrap();
        let elapsed = now.duration_since(last);

        if elapsed < Duration::from_secs(1) {
            return;
        }

        if elapsed >= Duration::from_secs(1) {
            let fps = ((self.frame_count as f32) / elapsed.as_secs_f32()).round() as u32;
            self.frame_count = 0;
            self.last_update = Some(now);
            self.update_fps(fps);
        }
    }

    fn as_any(&self) -> &dyn Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }
}
