//! # View Trait and Actions
//!
//! Defines the `View` trait that all views must implement, and the `ViewAction`
//! enum that represents actions views can request from the application layer.

use std::collections::HashSet;

use fps_config::Config;
use fps_levels::maze::Maze;
use fps_ui::{ComponentUpdate, UIEvent, manager::Layer};

/// High-level actions that views can request from the application.
///
/// These actions are processed by the `App` struct to coordinate between
/// views, game logic, and networking.
#[derive(Debug, Clone)]
pub enum ViewAction {
    /// Request to switch to another view by ID.
    SwitchTo(String),
    
    /// Request to exit the application.
    QuitApp,
    
    /// Custom action with string payload (for extensibility).
    Custom(String),
    
    /// Request UI component updates.
    UpdateComponent(Vec<ComponentUpdate>),
    
    /// Save the username to configuration.
    SaveUsername(String),
    
    /// Save a maze with the given name and editor ID.
    SaveMaze(String, String),
    
    /// Request to host a game with the specified settings.
    HostGame {
        /// Name of the game session.
        game_name: String,
        /// Maze to use for the game.
        maze: Maze,
        /// Target score string (win condition).
        target_score: String,
    },
    
    /// Request to join a game at the specified server address.
    JoinGame(String),
    
    /// Request to leave the lobby and return to main menu.
    LeaveLobby,
    
    /// Send a chat message.
    SendChatMessage(String),
    
    /// Send a request to start the game (host only).
    SendStartGame,
    
    /// Notify that the game has started.
    StartGame,
    
    /// Player keyboard input state (set of pressed keys).
    PlayerKeyboardInput(HashSet<String>),
    
    /// Player mouse input state (left mouse button pressed).
    PlayerMouseInput(bool),
    
    /// Game ended with the specified winner ID.
    GameEnd(String),
}

/// Trait that all views must implement.
///
/// Views are responsible for defining their UI layout and handling user input.
/// They produce high-level actions that are processed by the application layer.
pub trait View {
    /// Returns the unique identifier for this view.
    ///
    /// This ID is used for view switching and should match the view IDs
    /// defined in `crate::app::view_ids::views`.
    fn id(&self) -> &str;

    /// Returns the UI layer that this view defines.
    ///
    /// The layer contains all UI components for this view. This method is
    /// typically called once when the view is registered with the application.
    fn layer(&self) -> Layer;

    /// Handles UI events and produces high-level actions.
    ///
    /// This method is called for each UI event received from the UI manager.
    /// Views should process relevant events and return actions to be handled
    /// by the application layer.
    ///
    /// # Arguments
    ///
    /// * `events` - The UI event to process
    ///
    /// # Returns
    ///
    /// A vector of actions to be processed by the application layer.
    fn handle_ui_events(&mut self, events: &UIEvent) -> Vec<ViewAction>;

    /// Called when the view is activated (becomes visible).
    ///
    /// This method is called whenever the view becomes active. It can be used
    /// to update labels, reset input fields, sync state with configuration, etc.
    ///
    /// # Arguments
    ///
    /// * `config` - The application configuration
    ///
    /// # Returns
    ///
    /// A vector of component updates to apply when activating the view.
    fn on_activate(&mut self, _config: &Config) -> Vec<ComponentUpdate> {
        Vec::new() // default: do nothing
    }
}
