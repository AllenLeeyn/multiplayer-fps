use pixels::{Error, Pixels, SurfaceTexture};
use winit::dpi::PhysicalSize;
use winit::event_loop::ActiveEventLoop;
use winit::window::{Window, WindowAttributes};

use super::UIManager;

/// The AppDriver (now UIRenderer) manages the Pixels rendering surface
/// and handles low-level window events like resizing.
pub struct AppDriver<'a> {
    window: Window,
    pixels: Pixels<'a>,
    logical_width: u32,
    logical_height: u32,
    scale_factor: f32,
    logical_cursor: Option<(f64, f64)>,
}

impl<'a> AppDriver<'a> {
    pub fn new(
        event_loop: &ActiveEventLoop,
        logical_width: u32,
        logical_height: u32,
        physical_width: u32,
        physical_height: u32,
    ) -> Result<Self, Error> {
        let attrs = WindowAttributes::default()
            .with_title("fps_ui Crate Test Window")
            .with_inner_size(PhysicalSize::new(physical_width, physical_height))
            .with_resizable(false);

        let window = event_loop.create_window(attrs).unwrap();

        let window_ref: &'static Window = unsafe { &*(&window as *const Window) };

        let physical_size = window.inner_size();
        let scale_factor = Self::compute_scale_factor(
            logical_width,
            logical_height,
            physical_size.width,
            physical_size.height,
        );

        let surface_texture =
            SurfaceTexture::new(physical_size.width, physical_size.height, window_ref);
        let pixels = Pixels::new(logical_width, logical_height, surface_texture)?;

        Ok(Self {
            window,
            pixels,
            logical_width,
            logical_height,
            scale_factor,
            logical_cursor: None,
        })
    }

    /// Compute uniform scale factor from logical -> physical
    fn compute_scale_factor(
        logical_w: u32,
        logical_h: u32,
        physical_w: u32,
        physical_h: u32,
    ) -> f32 {
        let scale_x = physical_w as f32 / logical_w as f32;
        let scale_y = physical_h as f32 / logical_h as f32;
        scale_x.min(scale_y)
    }

    /// Map logical coordinates to physical framebuffer coordinates
    pub fn logical_to_physical(&self, lx: f64, ly: f64) -> (u32, u32) {
        (
            (lx * self.scale_factor as f64).round() as u32,
            (ly * self.scale_factor as f64).round() as u32,
        )
    }

    /// Map physical mouse coordinates to logical coordinates
    pub fn physical_to_logical(&self, px: f64, py: f64) -> (f64, f64) {
        (px / self.scale_factor as f64, py / self.scale_factor as f64)
    }

    /// Handle resizing events from winit
    pub fn handle_winit_event(&mut self, event: &winit::event::WindowEvent) {
        use winit::event::WindowEvent;

        match event {
            WindowEvent::Resized(size) => {
                self.pixels.resize_surface(size.width, size.height).unwrap();

                self.scale_factor = Self::compute_scale_factor(
                    self.logical_width,
                    self.logical_height,
                    size.width,
                    size.height,
                );
            }

            WindowEvent::CursorMoved { position, .. } => {
                let (lx, ly) = self.physical_to_logical(position.x, position.y);
                self.logical_cursor = Some((lx, ly));
            }

            _ => {}
        }
    }

    pub fn render(&mut self, ui_manager: &mut UIManager) -> Result<(), Error> {
        let frame = self.pixels.frame_mut();
        ui_manager.draw(frame);
        self.pixels.render()?;
        Ok(())
    }

    pub fn logical_cursor(&self) -> Option<(f64, f64)> {
        self.logical_cursor
    }

    pub fn window(&self) -> &winit::window::Window {
        &self.window
    }
}
