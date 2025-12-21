use std::any::Any;
use std::fmt::{Debug, Formatter, Result};

use super::super::{
    Color, Component, ComponentUpdate, LayoutMetrics, Rect, UIEvent, UIMainContext, WindowEvent,
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
        let start_x = self.bounds.x.round() as usize;
        let start_y = self.bounds.y.round() as usize;

        const BYTES_PER_PIXEL: usize = 4;

        let draw_start_x = 0;
        let draw_start_y = 0;
        let (draw_end_x, draw_end_y) = context.layout.size_as_usize();
        let draw_width = draw_end_x;

        for y in draw_start_y..draw_end_y {
            for x in draw_start_x..draw_end_x {
                let color = match &self.source {
                    RenderSource::SolidColor(c) => *c,

                    RenderSource::Image(file_path) => {
                        if let Some(image_data) = context.get_image_data(file_path) {
                            const TILE_WIDTH: usize = 100;
                            const BYTES_PER_IMAGE_PIXEL: usize = 4;

                            let tile_x = (x - start_x) % TILE_WIDTH;
                            let tile_y = (y - start_y) % TILE_WIDTH;

                            let pixel_offset =
                                (tile_y * TILE_WIDTH + tile_x) * BYTES_PER_IMAGE_PIXEL;

                            if pixel_offset + 3 < image_data.len() {
                                Color::new(
                                    image_data[pixel_offset],
                                    image_data[pixel_offset + 1],
                                    image_data[pixel_offset + 2],
                                    image_data[pixel_offset + 3],
                                )
                            } else {
                                Color::RED
                            }
                        } else {
                            Color::MAGENTA
                        }
                    }

                    RenderSource::Function(f) => {
                        let norm_x = (x as f64 - self.bounds.x) / self.bounds.w;
                        let norm_y = (y as f64 - self.bounds.y) / self.bounds.h;

                        f(norm_x, norm_y)
                    }
                };

                let offset = (y * draw_width + x) * BYTES_PER_PIXEL;

                if offset + 3 < frame.len() {
                    frame[offset] = color.r;
                    frame[offset + 1] = color.g;
                    frame[offset + 2] = color.b;
                    frame[offset + 3] = color.a;
                }
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
