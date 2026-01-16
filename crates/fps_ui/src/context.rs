use super::{LayoutContext, Color};
use super::fonts::FontManager;

use std::{collections::HashMap, path::Path};

#[derive(Clone, Debug)]
pub struct Texture {
    pub width: u32,
    pub height: u32,
    pub pixels: Vec<Color>, // row-major
}

impl Texture {
    pub fn new(width: u32, height: u32, pixels: Vec<Color>) -> Self {
        assert_eq!(pixels.len(), (width * height) as usize);
        Self { width, height, pixels }
    }

    /// Get pixel at integer coordinates (x, y)
    pub fn get_pixel(&self, x: u32, y: u32) -> Color {
        let xi = x.min(self.width - 1) as usize;
        let yi = y.min(self.height - 1) as usize;
        self.pixels[yi * self.width as usize + xi]
    }

    /// Sample the color at normalized coordinates (u, v) in [0,1]
    pub fn sample(&self, u: f32, v: f32) -> Color {
        let x = ((u.clamp(0.0, 1.0) * self.width as f32) as usize).min(self.width as usize - 1);
        let y = ((v.clamp(0.0, 1.0) * self.height as f32) as usize).min(self.height as usize - 1);
        self.pixels[y * self.width as usize + x]
    }
}


/// UIMainContext holds application-wide, mostly static resources
/// that are required by the UIManager and individual components for rendering.
#[derive(Debug)]
pub struct UIMainContext {
    pub font_manager: FontManager,
    pub textures: HashMap<String, Texture>,
    pub layout: LayoutContext,
}

impl UIMainContext {
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

    /// Load a PNG/JPG texture from disk and store it in the context
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
    
    pub fn add_texture(&mut self, id: &str, texture: Texture) {
        self.textures.insert(id.to_string(), texture);
    }

    pub fn get_texture(&self, id: &str) -> Option<&Texture> {
        self.textures.get(id)
    }
}
