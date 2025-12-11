use std::fmt::{Debug, Formatter, Result};
use std::cmp::min;

use super::super::{
    Color,
    Bounds,
    UIMainContext,
    Component,
    ComponentUpdate,
    UIEvent,
    LayoutMetrics,
    WindowEvent
};
// --- Render Source Definition ---

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
    bounds: Bounds, 
    source: RenderSource, // Holds the flexible rendering data
    needs_redraw: bool,
}

impl Panel {
    pub fn new(id: String, source: RenderSource, bounds: Bounds) -> Self {
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

    fn bounds(&self) -> Bounds {
        self.bounds
    }

    fn layout_metrics(&self) -> LayoutMetrics {
        // Returns the default, absolute (0, 0) TopLeft metric as a placeholder.
        LayoutMetrics::default()
    }

    /// Panels typically do not respond to input.
    fn handle_input(&mut self, _event: &WindowEvent) -> Vec<UIEvent> {
        Vec::new() 
    }

    fn draw(&self, frame: &mut [u8], context: &mut UIMainContext, screen_width: u32, screen_height: u32) {
        // Calculate the actual integer pixel area the panel covers on the screen
        let start_x = self.bounds.x.round() as usize;
        let start_y = self.bounds.y.round() as usize;
        
        // Use min() to clip the panel bounds to the screen edges
        let end_x = (self.bounds.x + self.bounds.w).round() as usize;
        let end_y = (self.bounds.y + self.bounds.h).round() as usize;
        
        let draw_width = screen_width as usize;

        // The number of bytes per pixel for the Pixels buffer (RGBA)
        const BYTES_PER_PIXEL: usize = 4; // FIX 1: Pixels uses RGBA (4 bytes)

        // Clip the draw region to the actual buffer limits
        let draw_start_x = start_x.max(0);
        let draw_start_y = start_y.max(0);
        let draw_end_x = min(end_x, screen_width as usize);
        let draw_end_y = min(end_y, screen_height as usize);


        for y in draw_start_y..draw_end_y {
            for x in draw_start_x..draw_end_x {
                
                let color = match &self.source {
                    RenderSource::SolidColor(c) => *c,

                    RenderSource::Image(file_path) => {
                        if let Some(image_data) = context.get_image_data(file_path) {
                            // FIX 2: Simplified Tiling (assuming RGBA image data format)
                            // This simplistic tiling needs image metadata (w, h) from UIMainContext 
                            // for proper indexing, but for now we use a fixed size to avoid panic.
                            const TILE_WIDTH: usize = 100;
                            const BYTES_PER_IMAGE_PIXEL: usize = 4; // Assumed image format

                            let tile_x = (x - start_x) % TILE_WIDTH; // X coordinate relative to the panel, wrapped
                            let tile_y = (y - start_y) % TILE_WIDTH; // Y coordinate relative to the panel, wrapped

                            let pixel_offset = (tile_y * TILE_WIDTH + tile_x) * BYTES_PER_IMAGE_PIXEL;
                            
                            if pixel_offset + 3 < image_data.len() {
                                Color::new(
                                    image_data[pixel_offset],
                                    image_data[pixel_offset + 1],
                                    image_data[pixel_offset + 2],
                                    image_data[pixel_offset + 3], // Read the Alpha channel
                                )
                            } else {
                                Color::RED // Error: Image access out of bounds
                            }
                        } else {
                            Color::MAGENTA // Error: Image not loaded
                        }
                    }
                    
                    RenderSource::Function(f) => {
                        // Calculate normalized coordinates (0.0 to 1.0) relative to the panel's size
                        let norm_x = (x as f64 - self.bounds.x) / self.bounds.w;
                        let norm_y = (y as f64 - self.bounds.y) / self.bounds.h;
                        
                        f(norm_x, norm_y) 
                    }
                };

                // FIX 3: Correct pixel offset calculation for RGBA (4 bytes/pixel)
                let offset = (y * draw_width + x) * BYTES_PER_PIXEL;

                if offset + 3 < frame.len() { // Check bounds for all 4 bytes
                    frame[offset] = color.r;
                    frame[offset + 1] = color.g;
                    frame[offset + 2] = color.b;
                    frame[offset + 3] = color.a; // Write the Alpha channel
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
                self.bounds = Bounds::new(0.0, 0.0, *width, *height);
                self.needs_redraw = true;
                true 
            }
            _ => false,
        }
    }
    
    // In panel.rs (assuming Component trait was updated)
    fn requires_redraw(&mut self) -> bool {
        if self.needs_redraw {
            self.needs_redraw = false; // Reset the flag immediately
            true
        } else {
            false
        }
    }

    fn get_text(&self) -> &str {
        &self.id
    }
}