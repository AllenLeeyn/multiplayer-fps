use pixels::{Pixels, SurfaceTexture, Error};
use winit::window::Window;

use crate::context::UIMainContext;
use crate::layers::UIManager; // <-- IMPORT UIManager from layers.rs

/// The AppDriver (now UIRenderer) manages the Pixels rendering surface 
/// and handles low-level window events like resizing.
pub struct AppDriver<'a> {
    pixels: Pixels<'a>,
    width: u32,
    height: u32,
}

impl<'a> AppDriver<'a> {
    /// Creates a new AppDriver by initializing the Pixels renderer.
    pub fn new(window: &'a Window) -> Result<Self, Error> {
        let size = window.inner_size();
        let width = size.width;
        let height = size.height;

        let surface_texture = SurfaceTexture::new(width, height, window);
        let pixels = Pixels::new(width, height, surface_texture)?;

        Ok(Self { pixels, width, height })
    }

    /// Handles low-level Winit events that affect the rendering surface.
    pub fn handle_winit_event(&mut self, event: &winit::event::WindowEvent) {
        use winit::event::WindowEvent;
        
        if let WindowEvent::Resized(size) = event {
            self.width = size.width;
            self.height = size.height;
            self.pixels.resize_surface(size.width, size.height).unwrap();
            self.pixels.resize_buffer(size.width, size.height).unwrap();
        }
    }

    /// Draws the UI state onto the frame buffer and presents it to the screen.
    pub fn render(&mut self, manager: &mut UIManager, context: &mut UIMainContext) -> Result<(), pixels::Error> {
        let frame = self.pixels.frame_mut();
        manager.draw(frame, context, self.width, self.height);
        self.pixels.render()?;
        Ok(())
    }

    pub fn dimensions(&self) -> (u32, u32) {
        (self.width, self.height)
    }

}