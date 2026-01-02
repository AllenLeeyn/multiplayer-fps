use fps_config::Config;
use fps_levels::maze::Maze;
use fps_ui::{ComponentUpdate, UIEvent, manager::Layer};

pub enum ViewAction {
    SwitchTo(String), // Request to switch to another view
    QuitApp,          // Request to exit the app
    Custom(String),   // Custom action with payload
    UpdateComponent(Vec<ComponentUpdate>),
    SaveUsername(String),
    SaveMaze(String, String),
    HostGame {
        game_name: String,
        maze: Maze,
        target_score: String,
    },
    JoinGame(String),
    LeaveLobby,
    SendChatMessage(String),
}

pub trait View {
    /// Unique ID for this view
    fn id(&self) -> &str;

    /// Returns the layer this view defines
    fn layer(&self) -> Layer;

    /// Handle UI events (from UIManager) and produce high-level actions
    fn handle_ui_events(&mut self, events: &UIEvent) -> Vec<ViewAction>;

    /// Called when the view is activated (shown). Can update labels, reset inputs, etc.
    fn on_activate(&mut self, _config: &Config) -> Vec<ComponentUpdate> {
        Vec::new() // default: do nothing
    }
}
