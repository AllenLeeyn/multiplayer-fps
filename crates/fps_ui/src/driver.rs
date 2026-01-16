//! # App Driver Module
//!
//! Manages the window, pixel buffer, and coordinate system transformations.
//! Bridges winit window events with the UI rendering system.

use pixels::{Error, Pixels, SurfaceTexture};
use winit::dpi::PhysicalSize;
use winit::event_loop::ActiveEventLoop;
use winit::window::{Window, WindowAttributes};

use super::UIManager;

/// Application driver managing window and rendering surface.
///
/// Handles the low-level integration between winit (window management) and
/// pixels (rendering surface). Manages coordinate transformations between
/// logical UI coordinates and physical pixel coordinates.
///
/// The driver maintains a logical coordinate system that is independent of
/// the physical window size, allowing the UI to scale appropriately.
pub struct AppDriver<'a> {
    window: Window,
    pixels: Pixels<'a>,
    logical_width: u32,
    logical_height: u32,
    scale_factor: f32,
    logical_cursor: Option<(f64, f64)>,
}

impl<'a> AppDriver<'a> {
    /// Creates a new application driver with the specified dimensions.
    ///
    /// # Arguments
    ///
    /// * `event_loop` - The active winit event loop
    /// * `logical_width` - Logical width of the UI canvas
    /// * `logical_height` - Logical height of the UI canvas
    /// * `physical_width` - Initial physical window width
    /// * `physical_height` - Initial physical window height
    ///
    /// # Returns
    ///
    /// A new `AppDriver` or an error if initialization fails.
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// use fps_ui::AppDriver;
    ///
    /// let driver = AppDriver::new(
    ///     &event_loop,
    ///     800,  // logical width
    ///     600,  // logical height
    ///     1600, // physical width
    ///     1200, // physical height
    /// )?;
    /// ```
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

    /// Computes a uniform scale factor from logical to physical coordinates.
    ///
    /// Uses the minimum of X and Y scaling to maintain aspect ratio.
    ///
    /// # Arguments
    ///
    /// * `logical_w` - Logical width
    /// * `logical_h` - Logical height
    /// * `physical_w` - Physical width
    /// * `physical_h` - Physical height
    ///
    /// # Returns
    ///
    /// The uniform scale factor.
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

    /// Converts logical coordinates to physical framebuffer coordinates.
    ///
    /// # Arguments
    ///
    /// * `lx` - Logical X coordinate
    /// * `ly` - Logical Y coordinate
    ///
    /// # Returns
    ///
    /// Physical pixel coordinates `(x, y)`.
    pub fn logical_to_physical(&self, lx: f64, ly: f64) -> (u32, u32) {
        (
            (lx * self.scale_factor as f64).round() as u32,
            (ly * self.scale_factor as f64).round() as u32,
        )
    }

    /// Converts physical mouse coordinates to logical coordinates.
    ///
    /// This is used to transform window-relative mouse positions into
    /// the UI's logical coordinate system.
    ///
    /// # Arguments
    ///
    /// * `px` - Physical X coordinate
    /// * `py` - Physical Y coordinate
    ///
    /// # Returns
    ///
    /// Logical coordinates `(x, y)`.
    pub fn physical_to_logical(&self, px: f64, py: f64) -> (f64, f64) {
        (px / self.scale_factor as f64, py / self.scale_factor as f64)
    }

    /// Handles window events from winit.
    ///
    /// Processes resize and cursor movement events, updating internal state
    /// and coordinate transformations as needed.
    ///
    /// # Arguments
    ///
    /// * `event` - The window event to process
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

    /// Renders the UI to the window.
    ///
    /// Draws all visible components to the pixel buffer and presents it to the screen.
    ///
    /// # Arguments
    ///
    /// * `ui_manager` - The UI manager containing all components to render
    ///
    /// # Returns
    ///
    /// `Ok(())` on success, or an error if rendering fails.
    pub fn render(&mut self, ui_manager: &mut UIManager) -> Result<(), Error> {
        let frame = self.pixels.frame_mut();
        ui_manager.draw(frame);
        self.pixels.render()?;
        Ok(())
    }

    /// Gets the current logical cursor position.
    ///
    /// # Returns
    ///
    /// The logical cursor position if available, `None` otherwise.
    pub fn logical_cursor(&self) -> Option<(f64, f64)> {
        self.logical_cursor
    }

    /// Gets a reference to the underlying window.
    ///
    /// Useful for window operations like setting cursor grab mode or visibility.
    ///
    /// # Returns
    ///
    /// A reference to the winit window.
    pub fn window(&self) -> &winit::window::Window {
        &self.window
    }
}
