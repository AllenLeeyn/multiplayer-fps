use std::error::Error;
use winit::application::ApplicationHandler;
use winit::dpi::PhysicalSize;
use winit::event::WindowEvent;
use winit::event_loop::{ActiveEventLoop, EventLoop};
use winit::window::{Window, WindowId, WindowAttributes};

// --- Imports from fps_ui library ---
use fps_ui::{
    AppDriver,
    UIMainContext,
    layers::UIManager,
    Color,
    components::panel::{Panel, RenderSource},
    geometry::Rect,
    layers::UILayer,
};

const WINDOW_WIDTH: u32 = 1200;
const WINDOW_HEIGHT: u32 = 900;

/// The central application struct that holds the necessary state.
struct App<'a> {
    window: Option<Window>,
    driver: Option<AppDriver<'a>>, // Carries the 'a lifetime
    manager: UIManager,
    context: UIMainContext,
}

// --------------------------------------------------------------------------
// This block implements the Winit behavior.
// FIX: Implementation must use the lifetime parameter <'a>.
// --------------------------------------------------------------------------
impl<'a> ApplicationHandler for App<'a> {
    
    /// Initializes the window and the AppDriver (Renderer).
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        if self.window.is_none() {
            println!("--- Initializing Window and UI Renderer ---");

            let attrs = WindowAttributes::default()
                .with_title("fps_ui Crate Test Window")
                .with_inner_size(PhysicalSize::new(WINDOW_WIDTH, WINDOW_HEIGHT))
                .with_resizable(false);

            let window = event_loop.create_window(attrs).unwrap();

            // When creating the driver, we must coerce the window borrow to 'a.
            // This is safe because Winit ensures the Window lives as long as the event loop runs.
            let window_ref: &'a Window = unsafe { 
                std::mem::transmute(&window)
            };
            
            match AppDriver::new(window_ref) {
                Ok(driver) => {
                    self.driver = Some(driver);
                    self.window = Some(window);
                }
                Err(e) => {
                    eprintln!("FATAL: Failed to initialize AppDriver/Pixels: {}", e);
                    event_loop.exit(); 
                }
            }
        }
    }

    /// Handles window events (Input and Redraw).
    fn window_event(&mut self, event_loop: &ActiveEventLoop, _id: WindowId, event: WindowEvent) {
        
        let driver = match self.driver.as_mut() {
            Some(d) => d,
            None => return, 
        };
        
        driver.handle_winit_event(&event);

        match event {
            WindowEvent::CloseRequested => {
                event_loop.exit();
            },
            WindowEvent::RedrawRequested => {
                if let Err(e) = driver.render(&self.manager, &self.context) { 
                    eprintln!("Pixels error during render: {:?}", e);
                    event_loop.exit();
                } 
            }
            _ => (),
        }
    }

    /// Requests continuous redraws.
    fn about_to_wait(&mut self, _event_loop: &ActiveEventLoop) {
        if let Some(window) = self.window.as_ref() {
             window.request_redraw(); 
        }
    }
}
// --------------------------------------------------------------------------


// --- Main Entry Point ---

// FIX: main must declare the lifetime 'static and constrain the App struct.
fn main() -> Result<(), Box<dyn Error>> {
    println!("--- Starting fps_ui Test Harness ---");

    let event_loop = EventLoop::new()?;

    // Initialize UI Context and Manager (no lifetime issues here)
    let ui_context = UIMainContext::new("assets/fonts/Sono-Regular.ttf")?;
    let mut ui_manager = UIManager::new();
    let full_bounds = Rect::new(0.0, 0.0, WINDOW_WIDTH as f64, WINDOW_HEIGHT as f64);
    
    let background_panel = Panel::new(
        "main_bg".to_string(),
        RenderSource::SolidColor(Color::new(200, 20, 20, 255)), 
        full_bounds,
    );
    
    let background_layer = UILayer {
        id: "background".to_string(),
        z_index: 0,
        is_visible: true,
        is_modal: false,
        components: vec![Box::new(background_panel)],
    };

    ui_manager.add_layer(background_layer).expect("Failed to add background layer.");
    
    // FIX: Initialize App with the 'static lifetime.
    // We use a block and unsafe transmute to satisfy the type checker, 
    // which is unfortunately necessary when using Pixels/Winit together in this pattern.
    let mut app: App<'static> = App {
        window: None,       // Explicitly initialize Option<Window>
        driver: None,       // Explicitly initialize Option<AppDriver<'static>>
        manager: ui_manager, // Use the initialized value
        context: ui_context, // Use the initialized value
    };

    // Run the event loop
    // NOTE: Winit's run_app function handles the window ownership cleanly.
    event_loop.run_app(&mut app)?;

    println!("--- fps_ui Test Harness Exited Successfully ---");
    Ok(())
}