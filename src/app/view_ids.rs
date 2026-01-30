//! # View IDs Module
//!
//! Centralized string constants for view and component identifiers.
//! Prevents magic strings throughout the codebase and provides a single
//! source of truth for UI element names.

/// View identifier constants.
///
/// These match the view IDs returned by `View::id()` and are used
/// for switching between views.
pub mod views {
    /// Main menu view identifier.
    pub const MAIN_MENU: &str = "main_menu";
    
    /// Join game menu view identifier.
    pub const JOIN_MENU: &str = "join_menu";
    
    /// Host game menu view identifier.
    pub const HOST_MENU: &str = "host_menu";
    
    /// Level selection menu view identifier.
    pub const LEVEL_MENU: &str = "level_menu";
    
    /// Lobby view identifier.
    pub const LOBBY: &str = "lobby";
    
    /// In-game view identifier.
    pub const GAME: &str = "game";
}

/// Component identifier constants.
///
/// These match the component IDs used when creating UI components and
/// are used for updates via `ComponentUpdate`.
pub mod components {
    /// FPS counter component identifier.
    #[allow(dead_code)]
    pub const FPS_COUNTER: &str = "fps_counter";

    /// RTT (round-trip time) display label identifier.
    pub const RTT_LABEL: &str = "rtt_label";
    
    /// Username display label identifier.
    pub const USERNAME_LABEL: &str = "username_label";
    
    /// Lobby chat log text box identifier.
    pub const LOBBY_CHAT_LOG: &str = "lobby_chat_log";
    
    /// Lobby chat input field identifier.
    pub const LOBBY_CHAT_INPUT: &str = "lobby_chat_input";
    
    /// Lobby user list component identifier.
    pub const LOBBY_USER_LIST: &str = "lobby_user_list";
    
    /// Lobby server address label identifier.
    pub const LOBBY_ADDR_LABEL: &str = "lobby_addr_label";
    
    /// Lobby maze settings display identifier.
    pub const LOBBY_MAZE_SETTINGS: &str = "lobby_maze_settings";
    
    /// Lobby maze preview viewer identifier.
    pub const LOBBY_MAZE_VIEW: &str = "lobby_maze_view";
    
    /// Start game button identifier (host only).
    pub const START_GAME_BUTTON: &str = "start_game_button";
    
    /// Host game error message label identifier.
    pub const HOST_ERROR_LABEL: &str = "host_error_label";
    
    /// Join game error message label identifier.
    pub const JOIN_ERROR_LABEL: &str = "join_error_label";
    
    /// Save maze button identifier (level editor).
    pub const SAVE_MAZE_BUTTON: &str = "save_maze_button";
    
    /// Main game renderer component identifier.
    pub const GAME_RENDER: &str = "game_render";
    
    /// In-game minimap component identifier.
    pub const GAME_MINI_MAP: &str = "game_mini_map";
    
    /// In-game leaderboard text box identifier.
    pub const GAME_LEADERBOARD: &str = "game_leaderboard";
    
    /// In-game score display label identifier.
    pub const GAME_SCORE_LABEL: &str = "game_score_label";
}
