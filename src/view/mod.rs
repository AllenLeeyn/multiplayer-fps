//! # View Module
//!
//! Implements the view layer of the application. Views are UI screens that handle
//! user input and produce high-level actions. Each view defines its own UI components
//! and manages their state.
//!
//! ## Architecture
//!
//! Views implement the `View` trait which provides:
//! - `id()`: Unique identifier for the view
//! - `layer()`: UI layer definition with components
//! - `handle_ui_events()`: Processes UI events and produces `ViewAction`s
//! - `on_activate()`: Called when view becomes active (for initialization)
//!
//! ## Views
//!
//! - **ViewMainMenu**: Main menu with username input, join/host/level editor options
//! - **ViewJoinMenu**: Join game interface with server address input
//! - **ViewHostMenu**: Host game interface with maze configuration
//! - **ViewLevelMenu**: Level editor for creating custom mazes
//! - **ViewLobby**: Lobby view with chat, player list, and game settings
//! - **ViewGame**: In-game view with renderer, minimap, and HUD

pub mod view;
pub mod view_game;
pub mod view_host_menu;
pub mod view_join_menu;
pub mod view_level_menu;
pub mod view_lobby;
pub mod view_main_menu;

pub use view::{View, ViewAction};
pub use view_game::ViewGame;
pub use view_host_menu::ViewHostMenu;
pub use view_join_menu::ViewJoinMenu;
pub use view_level_menu::ViewLevelMenu;
pub use view_lobby::ViewLobby;
pub use view_main_menu::ViewMainMenu;
