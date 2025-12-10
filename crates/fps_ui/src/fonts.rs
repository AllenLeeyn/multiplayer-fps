use ab_glyph::{point, Font, FontArc, PxScale, ScaleFont, GlyphId};
use super::Color;
use std::fs::File;
use std::io::Read;
use std::path::Path;
use std::collections::HashMap;

/// Stores the cached RGBA pixel data for a single character at a specific size.
#[derive(Debug)]
pub struct CachedGlyph {
    pub pixels: Vec<u8>,
    pub width: u32,
    pub height: u32,
    pub x_offset: i32, // Horizontal offset needed for correct drawing position
    pub y_offset: i32, // Vertical offset
}

// Key for the cache: (GlyphId, font_size_u32)
type CacheKey = (GlyphId, u32, Color);

#[derive(Debug)]
pub struct FontManager {
    font: FontArc,
    cache: HashMap<CacheKey, CachedGlyph>,
}

impl FontManager {
    pub fn new<P: AsRef<Path>>(path: P) -> Result<Self, Box<dyn std::error::Error>> {
        let mut file = File::open(path.as_ref())?;
        let mut font_bytes = Vec::new();
        let _bytes_read = file.read_to_end(&mut font_bytes)?;

        let font = FontArc::try_from_vec(font_bytes)
            .map_err(|e| format!("Failed to parse font data: {:?}", e))?;

        Ok(Self { font, cache: HashMap::new() })
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

    /// Draws text using the character cache. Only the first time a character 
    /// is encountered at a given size is it rasterized (the expensive part).
    pub fn draw_text(
        &mut self, 
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
        let font_arc = &self.font;
        let scale = self.calculate_scale(size_pixels, global_scale);
        let scaled_font = font_arc.as_scaled(scale);
        
        let draw_width = screen_width as usize;
        const BPP: usize = 4;

        let base_line = start_y as f32 + scaled_font.ascent();
        let mut caret_x = start_x as f32;
        
        let fallback_advance = scaled_font.height() * 0.5;
        let mut previous_glyph_id = None;
        
        // Use a key derived from size to check the cache efficiently
        let key_size = (size_pixels * 100.0) as u32;

        for current_char in text.chars() {
            
            let current_glyph_id = font_arc.glyph_id(current_char);
            
            if current_char == ' ' {
                // Space handling is necessary outside the cache
                caret_x += scaled_font.line_gap().max(fallback_advance);
                previous_glyph_id = None; 
                continue;
            }

            // 1. Kerning
            if let Some(prev_id) = previous_glyph_id {
                caret_x += scaled_font.kern(prev_id, current_glyph_id);
            }
            
            // 2. Cache Lookup/Generation (The optimization happens here)
            let cache_key = (current_glyph_id, key_size, color);
            let cached_glyph = self.cache.entry(cache_key).or_insert_with(|| {
                // Cache miss: Generate the data using the immutable helper
                rasterize_glyph_data(font_arc, scale, current_glyph_id, color)
            });
            
            // 3. Positioning and Copy (FAST)
            
            // Absolute position on screen, adjusted by the cached glyph's internal offset
            let glyph_x = (caret_x + cached_glyph.x_offset as f32).round() as u32;
            let glyph_y = (base_line + cached_glyph.y_offset as f32).round() as u32;
            
            // Copy the cached pixels directly onto the frame buffer
            for y in 0..cached_glyph.height {
                for x in 0..cached_glyph.width {
                    
                    let abs_x = glyph_x + x;
                    let abs_y = glyph_y + y - size_pixels as u32;

                    if abs_x >= screen_width || abs_y >= screen_height { continue; }

                    let cache_offset = (y as usize * cached_glyph.width as usize + x as usize) * BPP;
                    let frame_offset = (abs_y as usize * draw_width + abs_x as usize) * BPP;

                    // Direct opaque copy (fastest possible)
                    if cached_glyph.pixels[cache_offset + 3] == 255 {
                        frame[frame_offset..frame_offset + BPP]
                             .copy_from_slice(&cached_glyph.pixels[cache_offset..cache_offset + BPP]);
                    }
                }
            }

            // 4. Advance Caret (using horizontal advance from the scaled font, not the bounding box)
            let advance = scaled_font.h_advance(current_glyph_id);
            caret_x += advance.max(fallback_advance);
            previous_glyph_id = Some(current_glyph_id);
        }
    }

    /// Measures the total pixel width of a string for a given font size and scale.
    pub fn measure_text_width(&self, text: &str, size_pixels: f32, global_scale: f32) -> f64 {
        let font_arc = &self.font;
        let scale = self.calculate_scale(size_pixels, global_scale);
        let scaled_font = font_arc.as_scaled(scale);
        
        let mut total_width = 0.0;
        let fallback_advance = scaled_font.height() * 0.5;
        let mut previous_glyph_id = None;
        
        for current_char in text.chars() {
            let current_glyph_id = font_arc.glyph_id(current_char);
            
            // 1. Kerning
            if let Some(prev_id) = previous_glyph_id {
                total_width += scaled_font.kern(prev_id, current_glyph_id);
            }
            
            // 2. Advance (The scaled font handles the advance for a space character)
            let advance = scaled_font.h_advance(current_glyph_id);
            total_width += advance.max(fallback_advance);
            previous_glyph_id = Some(current_glyph_id);
        }
        total_width as f64
    }
    
}

/// Performs expensive layout and rasterization for a single glyph and caches the result.
pub fn rasterize_glyph_data(
    font_arc: &FontArc,
    scale: PxScale,
    glyph_id: GlyphId,
    color: Color,
) -> CachedGlyph {
    
    const BPP: usize = 4;

    let scaled_font = font_arc.as_scaled(scale);
    let glyph = glyph_id.with_scale_and_position(scale, point(0.0, scaled_font.ascent()));
    
    let outlined_glyph = match font_arc.outline_glyph(glyph) { 
        Some(g) => g,
        None => return CachedGlyph { pixels: Vec::new(), width: 0, height: 0, x_offset: 0, y_offset: 0 },
    };

    let px_bounds = outlined_glyph.px_bounds();
    
    // The integer pixel origin of the glyph relative to the layout origin (0,0)
    let draw_origin_x = px_bounds.min.x.floor() as i32;
    let draw_origin_y = px_bounds.min.y.floor() as i32;
    
    let buffer_width = (px_bounds.max.x - px_bounds.min.x).ceil() as u32;
    let buffer_height = (px_bounds.max.y - px_bounds.min.y).ceil() as u32;

    let total_size = (buffer_width * buffer_height * BPP as u32) as usize;
    let mut pixels: Vec<u8> = vec![0; total_size]; 

    // RASTERIZATION (Anti-Aliased)
    outlined_glyph.draw(|rel_x, rel_y, alpha| {
        // 1. Calculate buffer coordinates using the fractional offset
        let x = rel_x as usize; 
        let y = rel_y as usize;

        let offset = (y * buffer_width as usize + x) * BPP;
        
        if offset + 3 < pixels.len() {

            // Set the color and the calculated alpha (A-A)
            pixels[offset]     = color.r;
            pixels[offset + 1] = color.g;
            pixels[offset + 2] = color.b;
            pixels[offset + 3] = (alpha * 255.0) as u8; // Use scaled alpha
        }
    });

    // Return the struct
    CachedGlyph {
        pixels,
        width: buffer_width,
        height: buffer_height,
        x_offset: draw_origin_x,
        y_offset: draw_origin_y,
    }
}
