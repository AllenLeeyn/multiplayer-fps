use super::LayoutContext;
use super::fonts::FontManager;

use std::{collections::HashMap, path::Path};

/// UIMainContext holds application-wide, mostly static resources
/// that are required by the UIManager and individual components for rendering.
#[derive(Debug)]
pub struct UIMainContext {
    pub font_manager: FontManager,
    pub image_cache: HashMap<String, Vec<u8>>,
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
            image_cache: HashMap::new(),
            layout,
        })
    }

    /// Checks if an image is currently loaded and cached.
    pub fn has_image(&self, path: &str) -> bool {
        self.image_cache.contains_key(path)
    }

    /// Retrieves a read-only slice of the cached image data for a given path.
    pub fn get_image_data(&self, path: &str) -> Option<&[u8]> {
        self.image_cache.get(path).map(|v| v.as_slice())
    }

    /// Adds new image data to the cache.
    pub fn add_image(&mut self, path: String, data: Vec<u8>) {
        self.image_cache.insert(path, data);
    }
}
