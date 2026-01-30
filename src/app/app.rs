//! # Application Module
//!
//! Main application struct that coordinates the UI, views, game logic, and networking.
//! Implements `ApplicationHandler` for winit event loop integration.
//!
//! The `App` manages:
//! - UI rendering and component updates
//! - View lifecycle and transitions
//! - Game server (when hosting)
//! - Game client (when joining)
//! - Input handling and game state
//! - Configuration management

use crate::app::GameState;
use crate::view::{View, ViewAction};
use std::net::SocketAddr;
use std::sync::mpsc;
use std::thread;
use std::collections::HashMap;
use std::time::Instant;

use fps_levels::config::MazeSize;
use winit::application::ApplicationHandler;
use winit::event_loop::ActiveEventLoop;
use winit::window::WindowId;
use winit::event::DeviceEvent;

use super::{GameServer, ServerHandle, game_client::Game, GameInputState};
use super::view_ids::{views, components};
use super::constants::{server, client};
use fps_config::Config;
use fps_levels::maze::Maze;
use fps_ui::{Rect, AppDriver, ComponentUpdate, WindowEvent, components::MazeEditor, manager::UIManager};

use crate::{LOGICAL_HEIGHT, LOGICAL_WIDTH, PHYSICAL_HEIGHT, PHYSICAL_WIDTH};

/// Main application struct managing the entire application lifecycle.
///
/// Coordinates between UI rendering, views, game logic, and networking.
/// Implements winit's `ApplicationHandler` trait for event loop integration.
pub struct App {
    pub driver: Option<AppDriver<'static>>,
    pub manager: UIManager,

    pub views: HashMap<String, Box<dyn View>>,
    pub active_view: Option<String>,

    pub config: Config,
    pub config_path: String,

    pub server: Option<ServerHandle>,
    pub game: Option<Game>,
    pub game_input: GameInputState,
    
    /// Last time input was sent to the server.
    pub last_input_send: Option<Instant>,
}

impl App {
    /// Registers a view with the application.
    ///
    /// Adds the view's layer to the UI manager and stores the view
    /// for later activation.
    ///
    /// # Arguments
    ///
    /// * `view` - The view to register (boxed dynamic view)
    pub fn register_view(&mut self, view: Box<dyn View>) {
        let view_id = view.id().to_string();

        // Register layer immediately
        let layer = view.layer();
        self.manager
            .add_layer(layer)
            .expect("Failed to add view layer");

        self.views.insert(view_id, view);
    }

    /// Activates a view by ID.
    ///
    /// Hides the current view, shows the new view, manages cursor lock
    /// for game views, and calls the view's `on_activate` callback.
    ///
    /// # Arguments
    ///
    /// * `view_id` - The ID of the view to activate
    pub fn activate_view(&mut self, view_id: &str) {
        // Hide current
        if let Some(current) = &self.active_view {
            self.manager.set_layer_visibility(current, false);
        }

        // Show new
        self.manager.set_layer_visibility(view_id, true);

        // Cursor handling
        if let Some(driver) = self.driver.as_ref() {
            let window = driver.window();

            if view_id == views::GAME {
                lock_cursor(window);
                self.manager
                    .set_hovered_component(Some(components::GAME_RENDER.to_string()));
                self.manager
                    .set_focused_component(Some(components::GAME_RENDER.to_string()));
            } else {
                unlock_cursor(window);
            }
        }

        // Update view components on activation
        if let Some(view) = self.views.get_mut(view_id) {
            let updates = view.on_activate(&self.config);
            if !updates.is_empty() {
                self.manager.apply_updates(updates);
            }
        }

        self.active_view = Some(view_id.to_string());
    }

    fn handle_view_action(&mut self, action: ViewAction, event_loop: &ActiveEventLoop) {
        match action {
            ViewAction::UpdateComponent(updates) => {
                self.manager.apply_updates(updates);
            }

            ViewAction::SwitchTo(view_id) => {
                self.activate_view(&view_id);
            }

            ViewAction::QuitApp => {
                self.kill_game();
                event_loop.exit();
            }

            ViewAction::SaveUsername(username) => {
                self.save_user(&username);
            }

            ViewAction::SaveMaze(maze_name, maze_id) => {
                self.save_maze(maze_name, maze_id);
            }

            ViewAction::SaveServer(address, alias) => {
                self.save_server(address, alias);
            }

            ViewAction::HostGame {
                game_name,
                maze,
                target_score,
            } => {
                self.host_game(game_name, maze, target_score);
            }

            ViewAction::JoinGame(server_addr) => {
                self.join_game(server_addr);
            }

            ViewAction::LeaveLobby => {
                self.kill_game();
                self.manager.apply_updates(vec![ComponentUpdate::SetText(
                    components::RTT_LABEL.into(),
                    "RTT: --".into(),
                )]);
                self.activate_view(views::MAIN_MENU);
            }

            ViewAction::SendChatMessage(chat_msg) => {
                if let Some(game) = &mut self.game {
                    let _ = game.send_chat_msg(&chat_msg, &self.config.username);
                    self.manager.apply_updates(vec![ComponentUpdate::SetText(
                        components::LOBBY_CHAT_INPUT.into(),
                        String::new(),
                    )]);
                }
            }

            ViewAction::SendStartGame => {
                self.send_start_game();
            }

            ViewAction::StartGame => {
                self.activate_view(views::GAME);
            }

            ViewAction::GameEnd(winner) => {
                self.game_input.clear();
                self.activate_view(views::LOBBY);
                self.manager.apply_updates(vec![ComponentUpdate::AppendTextVec(
                    components::LOBBY_CHAT_LOG.into(),
                    vec![format!("[{} WINS]", winner)]
                )]);
            }

            _ => {}
        }
    }

    fn kill_game(&mut self) {
        if let Some(game) = self.game.take() {
            println!("Disconnected from game '{}'", game.game_name);
            game.kill();
        }

        if let Some(server) = self.server.take() {
            server.shutdown();
        }
    }

    fn save_user(&mut self, mut username: &str) {
        if username.is_empty() {
            username = "unknown";
        }

        // Update config
        self.config.username = username.to_string();

        // Update the label
        self.manager.apply_updates(vec![ComponentUpdate::SetText(
            components::USERNAME_LABEL.into(),
            format!("Current user: {}", username),
        )]);

        // Save to disk
        self.config.save(&self.config_path);
    }

    fn save_maze(&mut self, maze_name: String, maze_id: String) {
        if let Some(editor) = self.manager.get_component_mut::<MazeEditor>(&maze_id) {
            let mut maze = editor.maze().clone(); // clone current maze
            maze.name = maze_name.clone(); // set new name

            // --- Overwrite stored maze ---
            self.config.maze = Some(maze.clone());

            // --- Save config ---
            self.config.save(&self.config_path);

            // Update the label
            self.manager.apply_updates(vec![ComponentUpdate::SetText(
                components::SAVE_MAZE_BUTTON.into(),
                "SAVED".into(),
            )]);

            println!("Maze '{}' saved successfully", maze_name);
        } else {
            eprintln!("MazeEditor component not found");
        }
    }

    fn save_server(&mut self, address: String, alias: String) {
        // Validate inputs
        if address.trim().is_empty() {
            self.manager.apply_updates(vec![ComponentUpdate::SetText(
                "join_error_label".into(),
                "Error: Address cannot be empty".to_string(),
            )]);
            return;
        }

        let alias = if alias.trim().is_empty() {
            address.clone() // Use address as alias if no alias provided
        } else {
            alias.trim().to_string()
        };

        // Save server to config
        self.config.saved_servers.insert(alias.clone(), address.clone());

        // Save config to disk
        self.config.save(&self.config_path);

        // Refresh saved server buttons
        let mut updates = vec![ComponentUpdate::SetText(
            "join_error_label".into(),
            format!("Server '{}' saved successfully", alias),
        )];
        
        // Update saved server buttons (up to 5)
        let servers_vec: Vec<(&String, &String)> = self.config.saved_servers.iter().take(5).collect();
        for (index, (alias, _address)) in servers_vec.iter().enumerate() {
            let button_id = format!("saved_server_{}", index);
            let button_text = format!("📌 {}", alias);
            updates.push(ComponentUpdate::SetText(button_id, button_text));
        }
        
        // Hide unused buttons
        for index in servers_vec.len()..5 {
            let button_id = format!("saved_server_{}", index);
            updates.push(ComponentUpdate::SetText(button_id, String::new()));
        }
        
        self.manager.apply_updates(updates);

        println!("Server '{}' ({}) saved successfully", alias, address);
    }

    pub fn validate_game_settings(
        &self,
        game_name: &str,
        maze: &mut Maze,
        target_score: String,
    ) -> Result<(), String> {
        // Validate game name
        if game_name.trim().is_empty() {
            return Err("Game name cannot be empty".to_string());
        }

        // Validate target score
        let score = target_score
            .parse::<u32>()
            .map_err(|_| "Target score must be a number".to_string())?;
        use super::constants::scoring;
        if !(scoring::MIN_TARGET_SCORE..=scoring::MAX_TARGET_SCORE).contains(&score) {
            return Err(format!("Target score must be between {} and {}", scoring::MIN_TARGET_SCORE, scoring::MAX_TARGET_SCORE));
        }

        // Validate maze connectivity
        if !maze.is_connected() {
            return Err("Maze is not fully connected".to_string());
        }

        // Validate spawn points
        maze.set_spawn_points()
            .map_err(|e| format!("Failed to set spawn points: {}", e))?;

        Ok(())
    }

    /// Connects to a game server.
    ///
    /// Creates a new game client and connects to the specified server.
    /// Sets up the lobby UI after successful connection.
    ///
    /// # Arguments
    ///
    /// * `server_addr` - Address of the server to connect to
    ///
    /// # Returns
    ///
    /// `Ok(())` if connection succeeds, error message otherwise.
    pub fn connect_server(&mut self, server_addr: SocketAddr) -> Result<(), String> {
        if self.game.is_some() {
            return Err("Already connected to a game".into());
        }

        let username = self.config.username.clone();

        let game = Game::connect(
            server_addr,
            "0.0.0.0:0", // ephemeral client port
            client::CONNECTION_TIMEOUT,
            username,
        )
        .map_err(|e| format!("Failed to connect to server: {}", e))?;

        println!(
            "Connected to game '{:?}' at {}",
            game.game_name, server_addr
        );

        self.game = Some(game);
        Ok(())
    }

    fn start_game_server(
        &mut self,
        game_name: String,
        maze: Maze,
        target_score: String,
    ) -> Result<String, String> {
        match start_server(
            server::DEFAULT_BIND_ADDR,
            game_name,
            maze,
            target_score,
            self.config.username.to_string(),
        ) {
            Ok((handle, public_addr)) => {
                println!("Server started successfully {:?}", public_addr);
                self.server = Some(handle);
                self.manager.apply_updates(vec![ComponentUpdate::SetText(
                    components::LOBBY_ADDR_LABEL.into(),
                    public_addr.clone().into(),
                )]);
                Ok(public_addr)
            }
            Err(e) => Err(format!("Failed to start server: {}", e)),
        }
    }

    fn host_game(&mut self, game_name: String, maze: Maze, target_score: String) {
        let mut new_maze = maze.clone();

        if let Err(msg) = self.validate_game_settings(&game_name, &mut new_maze, target_score.clone()) {
            self.set_host_error(msg);
            return;
        }

        println!(
            "Hosting game '{}' with target score {} and maze '{}'",
            game_name, target_score, maze.name
        );

        // Start server if not already running
        if self.server.is_none() {
            match self.start_game_server(game_name, new_maze, target_score) {
                Ok(server_addr) => {
                    self.set_host_error("");
                    self.join_game(server_addr);
                }
                Err(e) => {
                    self.set_host_error(e);
                }
            }
        } else {
            // Server already running, get its address and join
            // Note: This case shouldn't normally happen, but handle it gracefully
            self.set_host_error("Server already running");
        }
    }

    fn set_host_error(&mut self, msg: impl Into<String>) {
        self.manager.apply_updates(vec![ComponentUpdate::SetText(
            components::HOST_ERROR_LABEL.into(),
            msg.into(),
        )]);
    }

    fn setup_lobby_ui(&mut self, maze: &Maze, target_score: u32, is_host: bool) {
        let mini_map_layout = get_mini_map_layout(maze.config.size);

        self.manager.apply_updates(vec![
            ComponentUpdate::SetText(
                components::LOBBY_MAZE_SETTINGS.into(),
                format!(
                    "{:?} {:?} Maze | Max Players: {} | Win Score: {}",
                    maze.config.size,
                    maze.config.difficulty,
                    maze.config.max_players(),
                    target_score,
                ),
            ),
            ComponentUpdate::SetVisibility(components::START_GAME_BUTTON.into(), is_host),
            ComponentUpdate::SetMaze(components::LOBBY_MAZE_VIEW.to_string(), maze.clone()),
            ComponentUpdate::SetMiniMapLayout(components::GAME_MINI_MAP.to_string(), mini_map_layout),
            ComponentUpdate::SetMaze(components::GAME_MINI_MAP.to_string(), maze.clone()),
            ComponentUpdate::SetMaze(components::GAME_RENDER.to_string(), maze.clone()),
        ]);
    }

    fn join_game(&mut self, server_addr: String) {
        self.set_host_error("Connecting... Please wait");
        self.set_join_error("Connecting... Please wait");

        let server_addr: SocketAddr = match server_addr.parse() {
            Ok(addr) => addr,
            Err(_) => {
                self.set_join_error("Invalid server address");
                return;
            }
        };

        println!("Joining game at {}", server_addr);

        if let Err(err) = self.connect_server(server_addr) {
            self.set_host_error(err.clone());
            self.set_join_error(err);
            return;
        }

        self.set_host_error("");
        self.set_join_error("");
        self.activate_view(views::LOBBY);

        if let Some(game) = self.game.as_ref() {
            let game_state = game.state;
            let maze = game.maze.clone();
            let target_score = game.target_score;
            let is_host = game.is_host;
            
            self.setup_lobby_ui(&maze, target_score, is_host);

            if game_state == GameState::InGame {
                self.activate_view(views::GAME);
            }
        } else {
            let error_msg = "Game not found".to_string();
            self.set_host_error(error_msg.clone());
            self.set_join_error(error_msg);
        }
    }

    fn set_join_error(&mut self, msg: impl Into<String>) {
        self.manager.apply_updates(vec![ComponentUpdate::SetText(
            components::JOIN_ERROR_LABEL.into(),
            msg.into(),
        )]);
    }

    fn send_start_game(&mut self) {
        if let Some(game) = self.game.as_mut() {
            match game.send_game_start() {
                Ok(_) => {}
                Err(e) => {
                    let msg = format!("Error: {:?}", e);
                    self.set_host_error(msg.clone());
                    self.set_join_error(msg);
                }
            }
        } else {
            self.set_host_error(format!("Game not found"));
            self.set_join_error(format!("Game not found"));
        }
    }
}

impl ApplicationHandler for App {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        println!("--- Initializing Window and UI Renderer ---");

        match AppDriver::new(
            event_loop,
            LOGICAL_WIDTH,
            LOGICAL_HEIGHT,
            PHYSICAL_WIDTH,
            PHYSICAL_HEIGHT,
        ) {
            Ok(driver) => {
                self.driver = Some(driver);
            }
            Err(e) => {
                eprintln!("FATAL: Failed to initialize AppDriver: {}", e);
                event_loop.exit();
            }
        }
    }

    fn device_event(
        &mut self,
        _event_loop: &ActiveEventLoop,
        _device_id: winit::event::DeviceId,
        event: DeviceEvent,
    ) {
        if let DeviceEvent::MouseMotion { delta } = event {
            if let Some(game) = &self.game && game.state == GameState::InGame {
                self.game_input.handle_mouse_motion(delta.0);
            }
        }
    }

    fn window_event(&mut self, event_loop: &ActiveEventLoop, _id: WindowId, event: WindowEvent) {
        let ui_events = {
            let driver = match self.driver.as_mut() {
                Some(d) => d,
                None => return,
            };

            self.manager.process_input(&event, driver.logical_cursor())
        };

        if let Some(game) = self.game.as_mut() {
            let (updates, actions) = game.poll();
            self.manager.apply_updates(updates);
            
            if game.state == GameState::InGame {
                self.game_input.handle_game_input(&event);
            }

            for action in actions {
                self.handle_view_action(action, event_loop);
            }
        }

        for ui_event in ui_events {
            let actions = match self.active_view.as_deref() {
                Some(id) => self
                    .views
                    .get_mut(id)
                    .map(|v| v.handle_ui_events(&ui_event))
                    .unwrap_or_default(),
                None => Vec::new(),
            };

            for action in actions {
                self.handle_view_action(action, event_loop);
            }
        }

        if let Some(driver) = self.driver.as_mut() {
            driver.handle_winit_event(&event);
        }

        match event {
            WindowEvent::CloseRequested => {
                self.kill_game();
                event_loop.exit();
            }

            WindowEvent::RedrawRequested => {
                if let Some(game) = self.game.as_mut() 
                    && game.state == GameState::InGame {
                        
                    // Check if enough time has passed since last input send (throttled to tick rate)
                    let should_send = match self.last_input_send {
                        Some(last) => last.elapsed() >= client::INPUT_SEND_INTERVAL,
                        None => true, // Send immediately on first frame
                    };
                    
                    if should_send {
                        let payload = self.game_input.snapshot(); // This clears the dx
                        let _ = game.send_game_input(payload);
                        self.last_input_send = Some(Instant::now());
                    }
                }

                self.manager.update_components();
                if let Some(driver) = self.driver.as_mut() {
                    driver.render(&mut self.manager).unwrap();
                }
            }

            WindowEvent::Focused(focused) => {
                if let Some(driver) = self.driver.as_ref() {
                    let window = driver.window();

                    if focused {
                        if self.active_view.as_deref() == Some(views::GAME) {
                            lock_cursor(window);
                        }
                    } else {
                        unlock_cursor(window);
                    }
                }
            }
            _ => {}
        }
    }

    fn about_to_wait(&mut self, _event_loop: &ActiveEventLoop) {
        if let Some(driver) = self.driver.as_ref() {
            driver.window().request_redraw();
        }
    }
}

fn start_server(
    bind_addr: &str,
    game_name: String,
    maze: Maze,
    target_score: String,
    host_username: String,
) -> Result<(ServerHandle, String), Box<dyn std::error::Error>> {
    let mut maze = maze.clone();
    maze.set_spawn_points()?;
    let server = GameServer::new(bind_addr, game_name, maze, target_score, host_username)?;

    let public_addr = server.public_addr()?;

    println!("{:?}", public_addr);
    let (cmd_tx, cmd_rx) = mpsc::channel();

    let join = thread::spawn(move || {
        if let Err(e) = std::panic::catch_unwind(|| server.run(cmd_rx)) {
            eprintln!("Server thread crashed: {:?}", e);
        }
    });

    Ok((ServerHandle { cmd_tx, join }, public_addr))
}

use winit::window::{CursorGrabMode, Window};

pub fn lock_cursor(window: &Window) -> bool {
    let grabbed = window
        .set_cursor_grab(CursorGrabMode::Locked)
        .or_else(|_| window.set_cursor_grab(CursorGrabMode::Confined))
        .is_ok();

    if grabbed {
        window.set_cursor_visible(false);
    }

    grabbed
}

/// Unlocks the cursor from the window.
///
/// # Arguments
///
/// * `window` - The window to unlock the cursor from
pub fn unlock_cursor(window: &Window) {
    let _ = window.set_cursor_grab(CursorGrabMode::None);
    window.set_cursor_visible(true);
}

/// Calculates layout metrics for the minimap component.
///
/// Returns the rectangle, scale factor, and offset for positioning
/// the minimap based on maze size.
///
/// # Arguments
///
/// * `size` - The size of the maze
///
/// # Returns
///
/// A tuple of `(rectangle, scale, offset)`.
pub fn get_mini_map_layout(size: MazeSize) -> (Rect, f32, f32) {
    let cell_px = 150/size.grid_size();
    let player_px = cell_px/2;
    let x = (LOGICAL_WIDTH as usize - cell_px * size.grid_size()) as f64;
    let rect = Rect::new(x, 0.0, 150.0, 150.0);

    println!("{:?}", (rect, cell_px as f32, player_px as f32));
    (rect, cell_px as f32, player_px as f32)
}