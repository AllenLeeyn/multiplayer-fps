use std::error::Error;

// We need to bring the necessary components from our own library's modules
use fps_ui::UIMainContext;
use fps_ui::UIManager;
use fps_ui::WinAttrs;
use fps_ui::run_event_loop;

// Make sure your UIManager and UIMainContext structures are accessible,
// likely by defining them in context.rs and layers.rs with default implementations
// if they aren't fully built yet.

fn main() -> Result<(), Box<dyn Error>> {
    println!("--- Starting fps_ui Test Harness ---");

    // 1. Initialize core UI state
    let ui_manager = UIManager::default();
    let ui_context = UIMainContext::default();

    // 2. Define Window Configuration
    let window_attributes = WinAttrs::default()
        .with_title("fps_ui Crate Test Window")
        .with_inner_size(winit::dpi::PhysicalSize::new(800, 600));

    // 3. Run the event loop (defined in window.rs)
    match run_event_loop(ui_manager, ui_context, window_attributes) {
        Ok(_) => {
            println!("--- fps_ui Test Harness Exited Successfully ---");
            Ok(())
        }
        Err(e) => {
            eprintln!("--- FATAL: Event Loop Error: {} ---", e);
            Err(e)
        }
    }
}
