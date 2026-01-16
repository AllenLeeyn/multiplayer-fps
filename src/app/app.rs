use crate::app::GameState;
use crate::view::{View, ViewAction};
use std::net::SocketAddr;
use std::sync::mpsc;
use std::thread;
use std::{collections::HashMap, time::Duration};

use fps_levels::config::MazeSize;
use winit::application::ApplicationHandler;
use winit::event_loop::ActiveEventLoop;
use winit::window::WindowId;
use winit::event::DeviceEvent;

use super::{GameServer, ServerHandle, game_client::Game, GameInputState};
use fps_config::Config;
use fps_levels::maze::Maze;
use fps_ui::{Rect, AppDriver, ComponentUpdate, WindowEvent, components::MazeEditor, manager::UIManager};

use crate::{LOGICAL_HEIGHT, LOGICAL_WIDTH, PHYSICAL_HEIGHT, PHYSICAL_WIDTH};

pub struct App {
    pub driver: Option<AppDriver<'static>>,
    pub manager: UIManager,

    pub views: HashMap<String, Box<dyn View>>,
    pub active_view: Option<String>,

    pub config: Config,
    pub config_path: String,

    pub server: Option<ServerHandle>,
    pub game: Option<Game>,
    pub game_input: GameInputState
}

impl App {
    pub fn register_view(&mut self, view: Box<dyn View>) {
        let view_id = view.id().to_string();

        // Register layer immediately
        let layer = view.layer();
        self.manager
            .add_layer(layer)
            .expect("Failed to add view layer");

        self.views.insert(view_id, view);
    }

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

            if view_id == "game" {
                lock_cursor(window);
                self.manager
                    .set_hovered_component(Some("game_render".to_string()));
                self.manager
                    .set_focused_component(Some("game_render".to_string()));
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
                self.activate_view("main_menu");
            }

            ViewAction::SendChatMessage(chat_msg) => {
                if let Some(game) = &mut self.game {
                    let _ = game.send_chat_msg(&chat_msg, &self.config.username);
                    self.manager.apply_updates(vec![ComponentUpdate::SetText(
                        "lobby_chat_input".into(),
                        String::new(),
                    )]);
                }
            }

            ViewAction::SendStartGame => {
                self.send_start_game();
            }

            ViewAction::StartGame => {
                self.activate_view("game");
            }

            ViewAction::GameEnd(winner) => {
                self.activate_view("lobby");
                self.manager.apply_updates(vec![ComponentUpdate::AppendTextVec(
                    "lobby_chat_log".into(),
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
            "username_label".into(),
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
                "save_maze_button".into(),
                "SAVED".into(),
            )]);

            println!("Maze '{}' saved successfully", maze_name);
        } else {
            eprintln!("MazeEditor component not found");
        }
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
        if !(30..=9999).contains(&score) {
            return Err("Target score must be between 30 and 9999".to_string());
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

    pub fn connect_server(&mut self, server_addr: SocketAddr) -> Result<(), String> {
        if self.game.is_some() {
            return Err("Already connected to a game".into());
        }

        let username = self.config.username.clone();

        let game = Game::connect(
            server_addr,
            "0.0.0.0:0", // ephemeral client port
            Duration::from_secs(5),
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

        // Start server
        let mut server_addr = String::new();
        if self.server.is_none() {
            match start_server(
                "0.0.0.0:9000",
                game_name,
                new_maze,
                target_score,
                self.config.username.to_string(),
            ) {
                Ok((handle, public_addr)) => {
                    println!("Server started successfully {:?}", public_addr);
                    self.server = Some(handle);
                    server_addr = public_addr.clone();
                    self.manager.apply_updates(vec![ComponentUpdate::SetText(
                        "lobby_addr_label".into(),
                        public_addr.into(),
                    )]);
                }
                Err(e) => {
                    self.set_host_error(format!("Failed to start server: {}", e));
                    return;
                }
            }
        }

        self.set_host_error("");
        self.join_game(server_addr.to_string());
    }

    fn set_host_error(&mut self, msg: impl Into<String>) {
        self.manager.apply_updates(vec![ComponentUpdate::SetText(
            "host_error_label".into(),
            msg.into(),
        )]);
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
        self.activate_view("lobby");

        if let Some(game) = self.game.as_ref() {
            let mini_map_layout = get_mini_map_layout(game.maze.config.size);

            self.manager.apply_updates(vec![
                ComponentUpdate::SetText(
                    "lobby_maze_settings".into(),
                    format!(
                        "{:?} {:?} Maze | Max Players: {} | Win Score: {}",
                        game.maze.config.size,
                        game.maze.config.difficulty,
                        game.maze.config.max_players(),
                        game.target_score,
                    ),
                ),
                ComponentUpdate::SetVisibility("start_game_button".into(), game.is_host),
                ComponentUpdate::SetMaze("lobby_maze_view".to_string(), game.maze.clone()),
                ComponentUpdate::SetMiniMapLayout("game_mini_map".to_string(), mini_map_layout),
                ComponentUpdate::SetMaze("game_mini_map".to_string(), game.maze.clone()),
                ComponentUpdate::SetMaze("game_render".to_string(), game.maze.clone()),
                
            ]);

            if game.state == GameState::InGame {
                self.activate_view("game");
            }

        } else {
            self.set_host_error(format!("Game not found"));
            self.set_join_error(format!("Game not found"));
        }
    }

    fn set_join_error(&mut self, msg: impl Into<String>) {
        self.manager.apply_updates(vec![ComponentUpdate::SetText(
            "join_error_label".into(),
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
                let payload = self.game_input.snapshot();
                let _ = game.send_game_input(payload);
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
                self.manager.update_components();
                if let Some(driver) = self.driver.as_mut() {
                    driver.render(&mut self.manager).unwrap();
                }
            }

            WindowEvent::Focused(focused) => {
                if let Some(driver) = self.driver.as_ref() {
                    let window = driver.window();

                    if focused {
                        if self.active_view.as_deref() == Some("game") {
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

pub fn unlock_cursor(window: &Window) {
    let _ = window.set_cursor_grab(CursorGrabMode::None);
    window.set_cursor_visible(true);
}

pub fn get_mini_map_layout(size: MazeSize) -> (Rect, f32, f32) {
    let cell_px = 150/size.grid_size();
    let player_px = cell_px/2;
    let x = (LOGICAL_WIDTH as usize - cell_px * size.grid_size()) as f64;
    let rect = Rect::new(x, 0.0, 150.0, 150.0);

    println!("{:?}", (rect, cell_px as f32, player_px as f32));
    (rect, cell_px as f32, player_px as f32)
}