use std::any::Any;
use std::fmt::{Debug, Formatter, Result};

use super::super::{
    Color, Component, ComponentUpdate, LayoutMetrics, Rect, UIEvent, UIMainContext, WindowEvent, draw_point
};

/// Defines the source and type of rendering for the Panel component.
pub enum RenderSource {
    /// Fills the area with a single solid color.
    SolidColor(Color),

    /// Renders an image loaded into memory. Holds a reference (ID) to the image data
    /// which lives in the UIMainContext.
    Image(String),

    /// Renders based on a custom function that generates pixel data dynamically.
    /// Args: (x, y) normalized coordinates relative to the panel's bounds (0.0 to 1.0).
    Function(Box<dyn Fn(f64, f64) -> Color + Send + Sync>),
}

impl Debug for RenderSource {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        match self {
            RenderSource::SolidColor(c) => f.debug_tuple("SolidColor").field(c).finish(),
            RenderSource::Image(path) => f.debug_tuple("Image").field(path).finish(),
            // The closure itself cannot be debugged, so we use a placeholder description.
            RenderSource::Function(_) => f.write_str("Function(<dynamic>)"),
        }
    }
}

// --- Panel Component Struct ---

/// A basic rectangular component used primarily for backgrounds or containers.
#[derive(Debug)]
pub struct Panel {
    id: String,
    bounds: Rect,
    source: RenderSource,
    needs_redraw: bool,
}

impl Panel {
    pub fn new(id: String, source: RenderSource, bounds: Rect) -> Self {
        Panel {
            id,
            bounds,
            source,
            needs_redraw: true,
        }
    }
}

// --- Component Trait Implementation ---

impl Component for Panel {
    fn id(&self) -> &str {
        &self.id
    }

    fn bounds(&self) -> Rect {
        self.bounds
    }

    fn layout_metrics(&self) -> LayoutMetrics {
        LayoutMetrics::default()
    }

    fn handle_input(
        &mut self,
        _event: &WindowEvent,
        _cursor_pos: Option<(f64, f64)>,
    ) -> Vec<UIEvent> {
        Vec::new()
    }

    fn draw(&self, frame: &mut [u8], context: &mut UIMainContext) {
        let draw_start_x = 0;
        let draw_start_y = 0;
        let (draw_width, draw_height) = context.layout.size_as_u32();

        for y in draw_start_y..draw_height {
            for x in draw_start_x..draw_width {
                let color = match &self.source {
                    RenderSource::SolidColor(c) => *c,
                    _ => Color::RED,
                };

                draw_point(frame, draw_width, draw_height, x, y, color);
            }
        }
    }

    fn apply_update(&mut self, update: &ComponentUpdate) -> bool {
        match update {
            ComponentUpdate::SetPosition(_, x, y) => {
                self.bounds.x = *x;
                self.bounds.y = *y;
                self.needs_redraw = true;
                true
            }
            ComponentUpdate::Resize { width, height } => {
                // Bounds::new is inclusive, so it creates the new Rect (0.0, 0.0, width, height)
                self.bounds = Rect::new(0.0, 0.0, *width, *height);
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

    fn get_text(&self) -> &str {
        &self.id
    }

    fn as_any(&self) -> &dyn Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }
}
