use ab_glyph::{point, Font, FontArc, PxScale, ScaleFont};
use super::Color;
use std::fs::File;
use std::io::Read;
use std::path::Path;
use std::iter::Peekable;
use std::str::Chars;

#[derive(Debug)]
pub struct FontManager {
    font: FontArc,
}

impl FontManager {
    pub fn new<P: AsRef<Path>>(path: P) -> Result<Self, Box<dyn std::error::Error>> {
        let mut file = File::open(path.as_ref())?;
        let mut font_bytes = Vec::new();
        let _bytes_read = file.read_to_end(&mut font_bytes)?;

        let font = FontArc::try_from_vec(font_bytes)
            .map_err(|e| format!("Failed to parse font data: {:?}", e))?;

        Ok(Self { font })
    }

    pub fn get_primary_font(&self) -> &FontArc {
        &self.font
    }

    pub fn calculate_scale(&self, size_pixels: f32, global_scale: f32) -> PxScale {
        let scaled_size = size_pixels * global_scale;
        PxScale {
            x: scaled_size,
            y: scaled_size,
        }
    }

    /// Draws a string of text onto the frame buffer using the primary font.
    ///
    /// This manually lays out glyphs by calculating the baseline and advancing the caret.
    pub fn draw_text(
        &self,
        frame: &mut [u8],
        text: &str,
        size_pixels: f32,
        color: Color,
        start_x: f64,
        start_y: f64,
        global_scale: f32,
        screen_width: u32,
        screen_height: u32,
    ) {
        let font = self.get_primary_font();
        let scale = self.calculate_scale(size_pixels, global_scale);
        
        let draw_width = screen_width as usize;
        const BYTES_PER_PIXEL: usize = 4; // RGBA

        let scaled_font = font.as_scaled(scale); 
        
        // Calculate the base line and start the caret position using f32 input
        let base_line = start_y as f32 + scaled_font.ascent();
        let mut caret_x = start_x as f32;
        
        let mut previous_glyph_id = None;
        let mut chars: Peekable<Chars<'_>> = text.chars().peekable();
        
        // Fallback for badly configured fonts (e.g., zero advance)
        let fallback_advance = scaled_font.height() * 0.5;

        while let Some(current_char) = chars.next() {
            
            // Handle space character explicitly (as the font's metric for space is often unreliable)
            if current_char == ' ' {
                caret_x += scaled_font.line_gap().max(fallback_advance);
                previous_glyph_id = None; 
                continue;
            }
            
            let current_glyph_id = font.glyph_id(current_char);

            // 1. Kerning (Adjustment before positioning)
            if let Some(prev_id) = previous_glyph_id {
                let kern_adjustment = scaled_font.kern(prev_id, current_glyph_id);
                caret_x += kern_adjustment;
            }
            
            // 2. Positioning
            let glyph = current_glyph_id.with_scale_and_position(scale, point(caret_x, base_line));
            
            // 3. Compute the outlined glyph for rasterization
            if let Some(outlined_glyph) = font.outline_glyph(glyph) {
                
                // --- FIX: Get the absolute pixel origin of the glyph ---
                let glyph_px_bounds = outlined_glyph.px_bounds();
                let glyph_origin_x = glyph_px_bounds.min.x.round() as u32;
                let glyph_origin_y = glyph_px_bounds.min.y.round() as u32;
                // --- End FIX ---
                
                // 4. Rasterize the glyph onto the frame buffer
                outlined_glyph.draw(|rel_x, rel_y, alpha| {
                    
                    // Calculate absolute screen coordinates by adding the origin
                    let x = glyph_origin_x + rel_x as u32;
                    let y = glyph_origin_y + rel_y as u32;

                    // Skip pixels outside the screen bounds
                    if x >= screen_width || y >= screen_height {
                        return;
                    }
                    
                    let offset = (y as usize * draw_width + x as usize) * BYTES_PER_PIXEL;
                    
                    if offset + 3 < frame.len() {
                        let alpha = alpha as f32;
                        
                        let src_r = color.r as f32;
                        let src_g = color.g as f32;
                        let src_b = color.b as f32;
                        
                        let dst_r = frame[offset] as f32;
                        let dst_g = frame[offset + 1] as f32;
                        let dst_b = frame[offset + 2] as f32;
                        
                        // Simple alpha blending (Source Over)
                        frame[offset]     = ((1.0 - alpha) * dst_r + alpha * src_r) as u8;
                        frame[offset + 1] = ((1.0 - alpha) * dst_g + alpha * src_g) as u8;
                        frame[offset + 2] = ((1.0 - alpha) * dst_b + alpha * src_b) as u8;
                    }
                });

                // 5. Advance Caret
                let advance = scaled_font.h_advance(current_glyph_id);
                // Use max() to ensure the caret always moves forward
                caret_x += advance.max(fallback_advance);

                previous_glyph_id = Some(current_glyph_id);
            }
        }
    }
}
