use pixels::{Pixels, SurfaceTexture};
use winit::application::ApplicationHandler;
use winit::event::WindowEvent;
use winit::event_loop::{ActiveEventLoop, ControlFlow, EventLoop};
pub use winit::window::{Window, WindowAttributes, WindowId};

use super::UIMainContext;

// --- Imports/Placeholders for your crate modules ---
pub struct UIRenderer;
impl UIRenderer {
    pub fn default() -> Self {
        Self
    }
}

/// The Application Driver.
/// This struct maintains the state of your application and handles the Winit events.
/// The Window and Pixels objects are stored separately to resolve self-reference errors.
pub struct AppDriver {
    /// The Winit Window handle, wrapped in Arc to facilitate surface borrowing.
    window: Option<Window>,

    /// The Pixels renderer, bound to the Window's lifetime (conceptually).
    pub pixels: Option<Pixels<'static>>,

    /// Your UI Manager (Logic)
    pub renderer: UIRenderer,

    /// Your Global Resources (Fonts, Style)
    pub context: UIMainContext,

    /// Pending configuration to be used when creating the window
    pub window_attrs: Option<WindowAttributes>,
}

impl ApplicationHandler for AppDriver {
    // 1. LIFECYCLE: App Resumed (Window Creation)
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        if self.window.is_none() {
            let attrs = self.window_attrs.take().unwrap_or_default();

            // --- Window Creation ---
            let window = match event_loop.create_window(attrs) {
                Ok(w) => w,
                Err(e) => {
                    eprintln!("Error creating window: {}", e);
                    event_loop.exit();
                    return;
                }
            };

            let size = window.inner_size();

            // 2. Create Pixels (Borrower)
            let surface_texture = unsafe {
                // 1. Get the raw reference pointer
                let raw_ptr = &window as *const Window;
                // 2. Transmute the pointer's lifetime to 'static
                let static_window_ref: &'static Window = &*raw_ptr;

                // 3. Use the 'static reference to create the surface texture
                SurfaceTexture::new(size.width, size.height, static_window_ref)
            };

            match Pixels::new(size.width, size.height, surface_texture) {
                Ok(p) => {
                    // 3. Store them separately in AppDriver fields
                    self.window = Some(window);
                    self.pixels = Some(p);
                }
                Err(e) => {
                    eprintln!("Error initializing pixels: {}", e);
                    event_loop.exit();
                }
            }
        }
    }

    // 2. LIFECYCLE: Window Events (Input -> Logic -> Render)
    fn window_event(&mut self, event_loop: &ActiveEventLoop, id: WindowId, event: WindowEvent) {
        // Safe unwraps: we rely on resumed() having run successfully
        let window = match self.window.as_ref() {
            Some(w) if w.id() == id => w,
            _ => return,
        };
        let pixels = self.pixels.as_mut().unwrap();

        match event {
            WindowEvent::CloseRequested => {
                println!("Close requested; stopping.");
                event_loop.exit();
            }

            WindowEvent::Resized(size) => {
                // Resize the Pixels surface texture
                if size.width > 0 && size.height > 0 {
                    pixels.resize_surface(size.width, size.height).unwrap();
                }
                window.request_redraw();
            }

            WindowEvent::RedrawRequested => {
                // --- RENDER CYCLE (The "Frame Tick") ---

                // 1. Render UI to the frame buffer
                // self.manager.render(pixels.frame_mut(), &self.context);

                // 2. Present the pixels to the screen
                if let Err(e) = pixels.render() {
                    eprintln!("Pixels render error: {}", e);
                    event_loop.exit();
                }

                // Request the next redraw to maintain a continuous loop (ControlFlow::Poll)
                window.request_redraw();
            }

            // --- INPUT HANDLING ---
            _ => {
                // self.manager.process_input(&event);
            }
        }
    }
}

/// The Entry Point for the UI crate.
/// Initializes the Winit EventLoop and runs the AppDriver.
pub fn run_event_loop(
    ui_renderer: UIRenderer,
    ui_context: UIMainContext,
    attrs: WindowAttributes,
) -> Result<(), Box<dyn std::error::Error>> {
    let event_loop = EventLoop::new()?;

    // Set ControlFlow::Poll for continuous game loops
    event_loop.set_control_flow(ControlFlow::Poll);

    let mut app = AppDriver {
        window: None,
        pixels: None,
        renderer: ui_renderer,
        context: ui_context,
        window_attrs: Some(attrs),
    };

    // Hand over control to Winit
    event_loop.run_app(&mut app)?;

    Ok(())
}
