use std::error::Error;
use winit::application::ApplicationHandler;
use winit::dpi::PhysicalSize;
use winit::event::WindowEvent;
use winit::event_loop::{ActiveEventLoop, EventLoop};
use winit::window::{Window, WindowId, WindowAttributes};
use std::time::Instant;

// --- Imports from fps_ui library ---
use fps_ui::{
    AppDriver,
    UIMainContext,
    layers::UIManager,
    Color,
    components::panel::{Panel, RenderSource},
    components::{Label, FpsComponent},
    events::{ComponentUpdate},
    geometry::Rect,
    layers::UILayer,
    layout::{AnchorPoint, LengthMode, LayoutMetrics},
};

const WINDOW_WIDTH: u32 = 1600;
const WINDOW_HEIGHT: u32 = 900;

/// The central application struct that holds the necessary state.
struct App<'a> {
    window: Option<Window>,
    driver: Option<AppDriver<'a>>, // Carries the 'a lifetime
    manager: UIManager,
    context: UIMainContext,

    last_fps_update: Instant,
    frame_count: u32,
    current_fps: u32,
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
                
                // 1. FPS Calculation and Update
                let now = Instant::now();
                self.frame_count += 1;

                // Update the FPS counter approximately once per second
                if now.duration_since(self.last_fps_update).as_secs_f32() >= 1.0 {
                    let duration = now.duration_since(self.last_fps_update).as_secs_f32();
                    let fps = (self.frame_count as f32 / duration).round() as u32;

                    self.current_fps = fps;
                    self.last_fps_update = now;
                    self.frame_count = 0;
                    
                    // --- Send Update to UIManager ---
                    let fps_update = ComponentUpdate::SetValue(
                        "fps_counter".to_string(), // Target ID of the FpsComponent
                        self.current_fps as f32,
                    );

                    // Construct the Vec<ComponentUpdate> and call the UIManager method
                    let updates = vec![fps_update];
                    self.manager.apply_updates(updates);
                }

                // 2. Render the UI
                if let Err(e) = driver.render(&mut self.manager, &mut self.context) { 
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
    let ui_context = UIMainContext::new("assets/fonts/VCR_OSD_MONO_1.001.ttf")?;
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

    // --- 2. Foreground Layer with Label ---

    // Define title label
    let label_bounds = Rect::new(0.11, 0.115, 100.0, 50.0); // x=20, y=20, max width=300, max height=50
    let title_label = Label::new(
        "title".to_string(),
        "aMAZE".to_string(), // Initial text
        360.0,                  // Font size in virtual pixels
        Color::new(255, 255, 255, 255), // White color
        label_bounds,
        LayoutMetrics {
            anchor: AnchorPoint::TopLeft,
            position_mode: LengthMode::Percentage,
            size_mode: LengthMode::AbsolutePixels,
        }
    );
    let label2_bounds = Rect::new(0.115, 0.125, 100.0, 50.0); // x=20, y=20, max width=300, max height=50
    let title2_label = Label::new(
        "title2".to_string(),
        "aMAZE".to_string(), // Initial text
        362.0,                  // Font size in virtual pixels
        Color::new(0, 0, 0, 255), // White color
        label2_bounds,
        LayoutMetrics {
            anchor: AnchorPoint::TopLeft,
            position_mode: LengthMode::Percentage,
            size_mode: LengthMode::AbsolutePixels,
        }
    );

    // FpsComponent
    let fps_bounds = Rect::new(10.0, 10.0, 150.0, 32.0); // Top-left corner, slightly offset
    let fps_component = FpsComponent::new(
        "fps_counter".to_string(), // Crucial ID for updates
        32.0,                      // Font size
        Color::new(255, 255, 0, 255), // Yellow color
        fps_bounds,
        LayoutMetrics {
            anchor: AnchorPoint::TopLeft,
            position_mode: LengthMode::AbsolutePixels,
            size_mode: LengthMode::AbsolutePixels,
        }
    );
    
    let foreground_layer = UILayer {
        id: "foreground".to_string(),
        z_index: 10,
        is_visible: true,
        is_modal: false,
        components: vec![
            Box::new(title2_label),
            Box::new(title_label),
            Box::new(fps_component),
        ],
    };

    ui_manager.add_layer(foreground_layer).expect("Failed to add foreground layer.");

    // --- 3. App Initialization and Run (Unchanged) ---
    let now = Instant::now();
    let mut app: App<'static> = App {
        window: None,
        driver: None,
        manager: ui_manager,
        context: ui_context,
        
        // --- FPS Tracking Initialization ---
        last_fps_update: now, // Initialize the timer start
        frame_count: 0,       // Initialize frame count
        current_fps: 0,       // Initialize displayed FPS
    };

    // Run the event loop
    event_loop.run_app(&mut app)?;

    println!("--- fps_ui Test Harness Exited Successfully ---");
    Ok(())
}