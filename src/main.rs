mod app;
pub mod view;

use std::collections::HashMap;
use std::error::Error;
use winit::event_loop::EventLoop;

// --- Imports from fps_ui library ---
use fps_config::Config;
use fps_ui::{
    Color, UIMainContext,
    components::{
        FpsComponent,
        panel::{Panel, RenderSource},
    },
    geometry::Rect,
    layout::{AnchorPoint, LayoutMetrics, LengthMode},
    manager::{Layer, UIManager},
};

use app::App;
use view::ViewMainMenu;

pub const PHYSICAL_WIDTH: u32 = 1600;
pub const PHYSICAL_HEIGHT: u32 = 900;
pub const LOGICAL_WIDTH: u32 = 800;
pub const LOGICAL_HEIGHT: u32 = 450;

fn main() -> Result<(), Box<dyn Error>> {
    println!("--- Starting fps_ui Test Harness ---");

    let config_path = "src/assets/config.json".to_string();
    let config = Config::load(&config_path);

    let event_loop = EventLoop::new()?;
    let ui_context = UIMainContext::new(
        "src/assets/fonts/8-bit-pusab.ttf",
        LOGICAL_WIDTH as f64,
        LOGICAL_HEIGHT as f64,
    )?;
    let mut ui_manager = UIManager::new(ui_context);

    // -------------------------------------------------
    // Background layer (always visible)
    // -------------------------------------------------
    let full_bounds = Rect::new(0.0, 0.0, LOGICAL_WIDTH as f64, LOGICAL_HEIGHT as f64);

    let background_panel = Panel::new(
        "main_bg".to_string(),
        RenderSource::SolidColor(Color::new(192, 0, 0, 255)),
        full_bounds,
    );

    let fps_component = FpsComponent::new(
        "fps_counter".to_string(),
        24.0,
        Color::YELLOW,
        Rect::new(2.0, 2.0, 50.0, 24.0),
        LayoutMetrics {
            anchor: AnchorPoint::TopLeft,
            positioning: LengthMode::Px,
            sizing: LengthMode::Px,
        },
    );

    let background_layer = Layer {
        id: "background".to_string(),
        z_index: 0,
        is_visible: true,
        is_modal: false,
        components: vec![Box::new(background_panel), Box::new(fps_component)],
    };

    ui_manager.add_layer(background_layer)?;

    // -------------------------------------------------
    // App + Views
    // -------------------------------------------------
    let main_menu_view = ViewMainMenu::new(config.username.clone());

    let mut app = App {
        window: None,
        driver: None,
        manager: ui_manager,
        views: HashMap::new(),
        active_view: None,
        config,
        config_path,
    };

    app.register_view(Box::new(main_menu_view));
    app.activate_view("main_menu");

    event_loop.run_app(&mut app)?;

    println!("--- fps_ui Test Harness Exited Successfully ---");
    Ok(())
}
