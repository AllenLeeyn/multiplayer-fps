//! # UI Context Module
//!
//! Manages global UI resources including fonts, textures, and layout context.
//! The `UIMainContext` is the central resource container passed to all components during rendering.

use super::{LayoutContext, Color};
use super::fonts::FontManager;

use std::{collections::HashMap, path::Path};

/// A texture containing pixel data for rendering.
///
/// Textures are stored in row-major order and can be sampled at integer or
/// normalized coordinates. Used for rendering images, sprites, and UI elements.
#[derive(Clone, Debug)]
pub struct Texture {
    pub width: u32,
    pub height: u32,
    pub pixels: Vec<Color>, // row-major
}

impl Texture {
    /// Creates a new texture from pixel data.
    ///
    /// # Arguments
    ///
    /// * `width` - Texture width in pixels
    /// * `height` - Texture height in pixels
    /// * `pixels` - Pixel data in row-major order (must be `width * height` in length)
    ///
    /// # Panics
    ///
    /// Panics if `pixels.len() != width * height`.
    pub fn new(width: u32, height: u32, pixels: Vec<Color>) -> Self {
        assert_eq!(pixels.len(), (width * height) as usize);
        Self { width, height, pixels }
    }

    /// Gets the pixel color at integer coordinates.
    ///
    /// Coordinates are clamped to valid texture bounds.
    ///
    /// # Arguments
    ///
    /// * `x` - X coordinate (0 to width-1)
    /// * `y` - Y coordinate (0 to height-1)
    ///
    /// # Returns
    ///
    /// The color at the specified coordinates.
    pub fn get_pixel(&self, x: u32, y: u32) -> Color {
        let xi = x.min(self.width - 1) as usize;
        let yi = y.min(self.height - 1) as usize;
        self.pixels[yi * self.width as usize + xi]
    }

    /// Samples the color at normalized coordinates.
    ///
    /// Coordinates are in the range [0.0, 1.0] where (0, 0) is the top-left
    /// and (1, 1) is the bottom-right. Values outside this range are clamped.
    ///
    /// # Arguments
    ///
    /// * `u` - Normalized X coordinate [0.0, 1.0]
    /// * `v` - Normalized Y coordinate [0.0, 1.0]
    ///
    /// # Returns
    ///
    /// The sampled color at the specified coordinates.
    pub fn sample(&self, u: f32, v: f32) -> Color {
        let x = ((u.clamp(0.0, 1.0) * self.width as f32) as usize).min(self.width as usize - 1);
        let y = ((v.clamp(0.0, 1.0) * self.height as f32) as usize).min(self.height as usize - 1);
        self.pixels[y * self.width as usize + x]
    }
}


/// Main UI context holding application-wide resources.
///
/// This context is shared across all UI components and provides access to:
/// - Font manager for text rendering
/// - Texture storage for images and sprites
/// - Layout context for coordinate calculations
///
/// The context is typically created once at application startup and passed
/// to the `UIManager` constructor.
#[derive(Debug)]
pub struct UIMainContext {
    pub font_manager: FontManager,
    pub textures: HashMap<String, Texture>,
    pub layout: LayoutContext,
}

impl UIMainContext {
    /// Creates a new UI context with the specified font and logical dimensions.
    ///
    /// # Arguments
    ///
    /// * `font_path` - Path to the TTF/OTF font file to load
    /// * `logical_width` - Logical width of the UI canvas
    /// * `logical_height` - Logical height of the UI canvas
    ///
    /// # Returns
    ///
    /// A new `UIMainContext` or an error if font loading fails.
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// use fps_ui::UIMainContext;
    ///
    /// let context = UIMainContext::new("fonts/main.ttf", 800.0, 600.0)?;
    /// ```
    pub fn new<P: AsRef<Path>>(
        font_path: P,
        logical_width: f64,
        logical_height: f64,
    ) -> Result<Self, Box<dyn std::error::Error>> {
        let font_manager = FontManager::new(font_path)?;
        let layout = LayoutContext::new(logical_width, logical_height);

        Ok(Self {
            font_manager,
            textures: HashMap::new(),
            layout,
        })
    }

    /// Loads a texture from a PNG/JPG image file.
    ///
    /// The texture is stored in the context and can be retrieved by ID.
    ///
    /// # Arguments
    ///
    /// * `id` - Unique identifier for the texture
    /// * `path` - File path to the image file
    ///
    /// # Returns
    ///
    /// `Ok(())` on success, or an error if the file cannot be loaded.
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// context.load_texture("button_bg", "assets/button.png")?;
    /// ```
    pub fn load_texture(&mut self, id: &str, path: &str) -> Result<(), Box<dyn std::error::Error>> {
        let img = image::open(Path::new(path))?.to_rgba8();
        let (width, height) = img.dimensions();

        let pixels: Vec<Color> = img
            .pixels()
            .map(|p| Color::new(p[0], p[1], p[2], p[3]))
            .collect();

        self.add_texture(id, Texture::new(width, height, pixels));
        Ok(())
    }
    
    /// Adds a texture to the context.
    ///
    /// # Arguments
    ///
    /// * `id` - Unique identifier for the texture
    /// * `texture` - The texture to store
    pub fn add_texture(&mut self, id: &str, texture: Texture) {
        self.textures.insert(id.to_string(), texture);
    }

    /// Retrieves a texture by ID.
    ///
    /// # Arguments
    ///
    /// * `id` - The texture identifier
    ///
    /// # Returns
    ///
    /// A reference to the texture if found, `None` otherwise.
    pub fn get_texture(&self, id: &str) -> Option<&Texture> {
        self.textures.get(id)
    }
}
