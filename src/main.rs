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

use app::{App, GameInputState};
use view::{ViewHostMenu, ViewJoinMenu, ViewLevelMenu, ViewLobby, ViewMainMenu, ViewGame};

pub const PHYSICAL_WIDTH: u32 = 1600;
pub const PHYSICAL_HEIGHT: u32 = 900;
pub const LOGICAL_WIDTH: u32 = 800;
pub const LOGICAL_HEIGHT: u32 = 450;

fn main() -> Result<(), Box<dyn Error>> {
    println!("--- Starting fps_ui Test Harness ---");

    let config_path = "src/assets/config.json".to_string();
    let config = Config::load(&config_path);

    let event_loop = EventLoop::new()?;
    let mut ui_context = UIMainContext::new(
        "src/assets/fonts/8-bit-pusab.ttf",
        LOGICAL_WIDTH as f64,
        LOGICAL_HEIGHT as f64,
    )?;

    ui_context.load_texture("floor", "src/assets/image/floor.png").expect("Failed to load floor texture");
    ui_context.load_texture("ceil", "src/assets/image/ceil.png").expect("Failed to load floor texture");
    ui_context.load_texture("wall", "src/assets/image/wall.png").expect("Failed to load wall texture");
    ui_context.load_texture("tile", "src/assets/image/tile.png").expect("Failed to load tile texture");
    ui_context.load_texture("tile_big", "src/assets/image/tile_big.png").expect("Failed to load tile_big texture");
    ui_context.load_texture("eye", "src/assets/image/eye.png").expect("Failed to load eye texture");

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
        16.0,
        Color::BLACK,
        Rect::new(2.0, 2.0, 50.0, 20.0),
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
        components: vec![Box::new(background_panel)],
    };


    let fps_layer = Layer {
        id: "fos".to_string(),
        z_index: 1000,
        is_visible: true,
        is_modal: false,
        components: vec![Box::new(fps_component)],
    };

    ui_manager.add_layer(background_layer)?;
    ui_manager.add_layer(fps_layer)?;

    // -------------------------------------------------
    // App + Views
    // -------------------------------------------------
    let main_menu_view = ViewMainMenu::new(config.username.clone());
    let join_game_view = ViewJoinMenu::new();
    let host_game_view = ViewHostMenu::new();
    let host_level_view = ViewLevelMenu::new();
    let lobby_view = ViewLobby::new();
    let game_view = ViewGame::new();

    let mut app = App {
        driver: None,
        manager: ui_manager,
        views: HashMap::new(),
        active_view: None,
        config,
        config_path,
        server: None,
        game: None,
        game_input: GameInputState::default(),
    };

    app.register_view(Box::new(main_menu_view));
    app.register_view(Box::new(join_game_view));
    app.register_view(Box::new(host_game_view));
    app.register_view(Box::new(host_level_view));
    app.register_view(Box::new(lobby_view));
    app.register_view(Box::new(game_view));
    app.activate_view("main_menu");

    event_loop.run_app(&mut app)?;

    println!("--- fps_ui Test Harness Exited Successfully ---");
    Ok(())
}
