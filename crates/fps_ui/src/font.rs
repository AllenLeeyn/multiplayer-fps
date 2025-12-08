use ab_glyph::{FontArc, PxScale};
use std::fs::File;
use std::io::Read;
use std::path::Path;

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
}
