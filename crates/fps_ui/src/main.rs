use std::error::Error;

// Assuming these imports work and are public in your library's entry point (lib.rs)
use fps_ui::UIMainContext;
use fps_ui::UIRenderer;
use fps_ui::WindowAttributes;
use fps_ui::run_event_loop;

// Assuming winit is available due to WindowAttributes/run_event_loop dependencies
// Use 'winit' for PhysicalSize
use winit::dpi::PhysicalSize;

fn main() -> Result<(), Box<dyn Error>> {
    println!("--- Starting fps_ui Test Harness ---");

    // 1. Initialize core UI context (must be first due to font loading)
    // NOTE: UIMainContext::new now requires the font path and returns a Result.
    let ui_context = match UIMainContext::new("assets/fonts/Sono-Regular.ttf") {
        Ok(context) => context,
        Err(e) => {
            eprintln!(
                "--- FATAL: Failed to initialize UIMainContext (Font Error: {}) ---",
                e
            );
            return Err(e);
        }
    };

    // 2. Initialize core UI state manager
    // The UIManager is decoupled and doesn't depend on the font.
    let ui_rendererr = UIRenderer::default();

    // 3. Define Window Configuration
    let window_attributes = WindowAttributes::default()
        .with_title("fps_ui Crate Test Window")
        .with_inner_size(PhysicalSize::new(1200, 900));

    // 4. Run the event loop (defined in window.rs)
    match run_event_loop(ui_rendererr, ui_context, window_attributes) {
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
