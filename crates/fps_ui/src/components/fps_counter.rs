use crate::Color;
use crate::components::Component;
use crate::context::UIMainContext;
use crate::events::{ComponentUpdate, UIEvent, UIInputEvent};
use crate::geometry::{Bounds, calculate_absolute_rect};
use crate::layout::LayoutMetrics;

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
    text: String, // The string being displayed (e.g., "FPS: 60")
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

    /// Updates the internal FPS value and refreshes the displayed text.
    /// Returns true if the text changed (requiring a redraw).
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
    
    fn handle_input(&mut self, _event: &UIInputEvent) -> Vec<UIEvent> {
        Vec::new()
    }

    /// Handles incoming updates from the application layer.
    fn apply_update(&mut self, update: &ComponentUpdate) -> bool {
        match update {
            ComponentUpdate::SetValue(id, new_value) if id == self.id() => {
                // The UIManager has correctly routed a SetValue update for this component.
                let new_fps = new_value.round() as u32;
                self.update_fps(new_fps) // Update FPS and return true if text changed
            }
            _ => false,
        }
    }

    fn requires_redraw(&mut self) -> bool {
        std::mem::take(&mut self.needs_redraw)
    }

    fn draw(&self, frame: &mut [u8], context: &UIMainContext, screen_width: u32, screen_height: u32) {
        let font_manager = &context.font_manager;
        
        let screen_w_f64 = screen_width as f64;
        let screen_h_f64 = screen_height as f64;

        // Calculate final absolute position
        let absolute_rect = calculate_absolute_rect(
            &self.layout_metrics,
            &self.bounds,
            screen_w_f64,
            screen_h_f64,
        );

        font_manager.draw_text(
            frame, 
            &self.text, // Use the dynamically updated text string
            self.font_size, 
            self.color, 
            absolute_rect.x, 
            absolute_rect.y, 
            context.global_scale_factor, 
            screen_width, 
            screen_height
        );
    }
}