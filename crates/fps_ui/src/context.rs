use super::font::FontManager;
use std::path::Path; // Import the FontManager from the sibling module

/// UIMainContext holds application-wide, mostly static resources
/// that are required by the UIManager and individual components for rendering.
///
/// This acts as the read-only global state container.
#[derive(Debug)]
pub struct UIMainContext {
    /// Manages the single loaded font asset. Guaranteed to have a font after construction.
    pub font_manager: FontManager,

    /// General scaling factor for DPI or resolution independence (e.g., 1.0, 1.5, 2.0).
    pub global_scale_factor: f32,
}

impl UIMainContext {
    /// Creates a new, initialized UIMainContext, requiring the path to the font file.
    ///
    /// This is the application's first opportunity to set up necessary global resources.
    ///
    /// P: The type that can be converted into a file path (e.g., String, &str).
    pub fn new<P: AsRef<Path>>(font_path: P) -> Result<Self, Box<dyn std::error::Error>> {
        // Attempt to initialize the FontManager by loading the font file.
        // This is where file I/O errors and font parsing errors are propagated.
        let font_manager = FontManager::new(font_path)?;

        Ok(UIMainContext {
            font_manager,
            global_scale_factor: 1.0, // Default to no scaling
        })
    }
}
