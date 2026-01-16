//! # Fonts Module
//!
//! Provides font loading, glyph caching, and text rendering functionality.
//! Uses ab_glyph for font parsing and implements a glyph cache for efficient
//! text rendering by avoiding repeated rasterization of the same characters.

use super::{Color, draw_point};
use ab_glyph::{Font, FontArc, GlyphId, PxScale, ScaleFont, point};
use std::collections::HashMap;
use std::fs::File;
use std::io::Read;
use std::path::Path;

/// Cached glyph data for a single character at a specific size and color.
///
/// Stores the pre-rasterized pixel data to avoid expensive re-rasterization
/// when the same character is drawn multiple times.
#[derive(Debug)]
pub struct CachedGlyph {
    pub pixels: Vec<u8>,
    pub width: u32,
    pub height: u32,
    pub x_offset: i32,
    pub y_offset: i32,
}

/// Cache key type: (GlyphId, quantized_size, Color)
type CacheKey = (GlyphId, u32, Color);

/// Manages font loading, glyph caching, and text rendering.
///
/// The font manager loads a single font file and provides efficient text
/// rendering with automatic glyph caching. Each unique character/size/color
/// combination is rasterized once and cached for subsequent use.
#[derive(Debug)]
pub struct FontManager {
    font: FontArc,
    cache: HashMap<CacheKey, CachedGlyph>,
}

impl FontManager {
    /// Creates a new font manager by loading a font file.
    ///
    /// # Arguments
    ///
    /// * `path` - Path to a TTF or OTF font file
    ///
    /// # Returns
    ///
    /// A new `FontManager` or an error if the font cannot be loaded.
    pub fn new<P: AsRef<Path>>(path: P) -> Result<Self, Box<dyn std::error::Error>> {
        let mut file = File::open(path.as_ref())?;
        let mut font_bytes = Vec::new();
        let _bytes_read = file.read_to_end(&mut font_bytes)?;

        let font = FontArc::try_from_vec(font_bytes)
            .map_err(|e| format!("Failed to parse font data: {:?}", e))?;

        Ok(Self {
            font,
            cache: HashMap::new(),
        })
    }

    /// Gets a reference to the primary font.
    ///
    /// Useful for advanced font operations that require direct font access.
    pub fn get_primary_font(&self) -> &FontArc {
        &self.font
    }

    /// Calculates the scale factor for a given font size.
    ///
    /// # Arguments
    ///
    /// * `size_pixels` - Desired font size in pixels
    ///
    /// # Returns
    ///
    /// A `PxScale` value for use with ab_glyph.
    pub fn calculate_scale(&self, size_pixels: f32) -> PxScale {
        let scaled_size = size_pixels;
        PxScale {
            x: scaled_size,
            y: scaled_size,
        }
    }

    /// Draws text to the frame buffer using cached glyphs.
    ///
    /// Text is rendered with automatic kerning, glyph caching, and proper
    /// baseline alignment. Characters are rasterized on first use and cached
    /// for subsequent draws.
    ///
    /// # Arguments
    ///
    /// * `frame` - The RGBA frame buffer to draw into
    /// * `text` - The text string to render
    /// * `size_pixels` - Font size in pixels
    /// * `color` - Text color
    /// * `start_x` - Starting X coordinate (logical)
    /// * `start_y` - Starting Y coordinate (logical)
    /// * `logical_width` - Logical canvas width (for bounds checking)
    /// * `logical_height` - Logical canvas height (for bounds checking)
    pub fn draw_text(
        &mut self,
        frame: &mut [u8],
        text: &str,
        size_pixels: f32,
        color: Color,
        start_x: f64,
        start_y: f64,
        logical_width: u32,
        logical_height: u32,
    ) {
        let font_arc = &self.font;
        let scale = self.calculate_scale(size_pixels);
        let scaled_font = font_arc.as_scaled(scale);

        let draw_width = logical_width as usize;
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
                    let abs_x = glyph_x + x as u32;
                    let abs_y = glyph_y + y as u32 - size_pixels as u32;

                    if abs_x >= logical_width || abs_y >= logical_height {
                        continue;
                    }

                    let cache_offset =
                        (y as usize * cached_glyph.width as usize + x as usize) * BPP;
                    let frame_offset = (abs_y as usize * draw_width + abs_x as usize) * BPP;

                    // Direct opaque copy (fastest possible)
                    if cached_glyph.pixels[cache_offset + 3] > 128 {
                        frame[frame_offset..frame_offset + BPP].copy_from_slice(
                            &cached_glyph.pixels[cache_offset..cache_offset + BPP],
                        );
                    }
                }
            }

            // 4. Advance Caret (using horizontal advance from the scaled font, not the bounding box)
            let advance = scaled_font.h_advance(current_glyph_id);
            caret_x += advance.max(fallback_advance);
            previous_glyph_id = Some(current_glyph_id);
        }
    }

    /// Measures the total pixel width of a text string.
    ///
    /// Useful for centering text or calculating layout requirements.
    /// Includes kerning adjustments for accurate measurements.
    ///
    /// # Arguments
    ///
    /// * `text` - The text to measure
    /// * `size_pixels` - Font size in pixels
    ///
    /// # Returns
    ///
    /// The total width in logical pixels.
    pub fn measure_text_width(&self, text: &str, size_pixels: f32) -> f64 {
        let font_arc = &self.font;
        let scale = self.calculate_scale(size_pixels);
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

/// Rasterizes a single glyph and returns cached data.
///
/// This is the expensive operation that converts a glyph outline into
/// pixel data. The result is cached by `FontManager` to avoid repeated
/// rasterization of the same character.
///
/// # Arguments
///
/// * `font_arc` - The font to use
/// * `scale` - Font scale factor
/// * `glyph_id` - The glyph identifier
/// * `color` - The color to render the glyph
///
/// # Returns
///
/// Cached glyph data with pixel buffer and positioning offsets.
pub fn rasterize_glyph_data(
    font_arc: &FontArc,
    scale: PxScale,
    glyph_id: GlyphId,
    color: Color,
) -> CachedGlyph {
    const BPP: usize = 4;
    let mut color = color.clone();

    let scaled_font = font_arc.as_scaled(scale);
    let glyph = glyph_id.with_scale_and_position(scale, point(0.0, scaled_font.ascent()));

    let outlined_glyph = match font_arc.outline_glyph(glyph) {
        Some(g) => g,
        None => {
            return CachedGlyph {
                pixels: Vec::new(),
                width: 0,
                height: 0,
                x_offset: 0,
                y_offset: 0,
            };
        }
    };

    let px_bounds = outlined_glyph.px_bounds();

    // The integer pixel origin of the glyph relative to the layout origin (0,0)
    let draw_origin_x = px_bounds.min.x.floor() as i32;
    let draw_origin_y = px_bounds.min.y.floor() as i32;

    let buffer_width = (px_bounds.max.x - px_bounds.min.x).ceil() as u32;
    let buffer_height = (px_bounds.max.y - px_bounds.min.y).ceil() as u32;

    let total_size = (buffer_width * buffer_height) as usize * BPP;
    let mut pixels: Vec<u8> = vec![0; total_size];

    // RASTERIZATION (Anti-Aliased)
    outlined_glyph.draw(|rel_x, rel_y, alpha| {
        color.set_alpha_f32(alpha);
        // Use draw_point
        draw_point(&mut pixels, buffer_width, buffer_height, rel_x, rel_y, color);
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
