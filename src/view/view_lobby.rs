//! # Lobby View
//!
//! View displayed in the game lobby. Shows game settings, connected players,
//! chat interface, and controls for starting the game (host only).

use super::{View, ViewAction};
use fps_config::Config;
use fps_levels::maze::Maze;

use fps_ui::{
    Color, ComponentUpdate, UIEvent,
    components::{Button, Label, MazeView, TextBox, TextInput},
    geometry::Rect,
    layout::{AnchorPoint, LayoutMetrics, LengthMode},
    manager::Layer,
};
use crate::app::view_ids::views;

/// Lobby view implementation.
pub struct ViewLobby {
    /// Current maze configuration.
    maze: Maze,
    
    /// Target score for winning the game.
    target_score: u32,

    /// List of connected user names.
    users: Vec<String>,
    
    /// Chat messages (for internal tracking).
    messages: Vec<String>,
    
    /// Current chat input text.
    chat_input: String,
}

impl ViewLobby {
    /// Creates a new lobby view.
    pub fn new() -> Self {
        Self {
            maze: Maze::default(),
            target_score: 0,

            users: Vec::new(),
            messages: Vec::new(),
            chat_input: String::new(),
        }
    }

    /// Attempts to send a chat message if the input is not empty.
    ///
    /// Clears the chat input and returns an action to send the message.
    fn try_send_chat(&mut self) -> Vec<ViewAction> {
        let text = self.chat_input.trim().to_string();
        self.chat_input.clear();

        if !text.is_empty() {
            vec![ViewAction::SendChatMessage(text)]
        } else {
            vec![]
        }
    }
}

impl View for ViewLobby {
    fn id(&self) -> &str {
        views::LOBBY
    }

    fn layer(&self) -> Layer {
        let title = Label::new(
            "lobby_title".to_string(),
            "Lobby".to_string(),
            64.0,
            Color::WHITE,
            Rect::new(0.05, 0.08, 1.0, 1.0),
            LayoutMetrics {
                anchor: AnchorPoint::TopLeft,
                positioning: LengthMode::Percent,
                sizing: LengthMode::Percent,
            },
        );

        let username_label4 = Label::new(
            "username_label4".to_string(),
            format!("as: unknown"),
            24.0,
            Color::WHITE,
            Rect::new(0.3, 0.1, 400.0, 10.0),
            LayoutMetrics {
                anchor: AnchorPoint::TopLeft,
                positioning: LengthMode::Percent,
                sizing: LengthMode::Px,
            },
        );

        let lobby_addr_label = Label::new(
            "lobby_addr_label".to_string(),
            "0.0.0.0::0".to_string(),
            16.0,
            Color::WHITE,
            Rect::new(0.3, 0.16, 1.0, 1.0),
            LayoutMetrics {
                anchor: AnchorPoint::TopLeft,
                positioning: LengthMode::Percent,
                sizing: LengthMode::Percent,
            },
        );

        let lobby_maze_label = Label::new(
            "lobby_maze_settings".to_string(),
            "-".to_string(),
            16.0,
            Color::WHITE,
            Rect::new(0.05, 0.2, 600.0, 30.0),
            LayoutMetrics {
                anchor: AnchorPoint::TopLeft,
                positioning: LengthMode::Percent,
                sizing: LengthMode::Px,
            },
        );

        let user_list = TextBox::new(
            "lobby_user_list".to_string(),
            Rect::new(0.05, 0.25, 160.0, 315.0),
            LayoutMetrics {
                anchor: AnchorPoint::TopLeft,
                positioning: LengthMode::Percent,
                sizing: LengthMode::Px,
            },
            16.0,
            Color::WHITE,
            Color::BLACK,
        );

        let chat_log = TextBox::new(
            "lobby_chat_log".to_string(),
            Rect::new(0.26, 0.25, 500.0, 280.0),
            LayoutMetrics {
                anchor: AnchorPoint::TopLeft,
                positioning: LengthMode::Percent,
                sizing: LengthMode::Px,
            },
            16.0,
            Color::WHITE,
            Color::BLACK,
        );

        let chat_input = TextInput::new(
            "lobby_chat_input".to_string(),
            Rect::new(0.26, 0.88, 400.0, 30.0),
            LayoutMetrics {
                anchor: AnchorPoint::TopLeft,
                positioning: LengthMode::Percent,
                sizing: LengthMode::Px,
            },
            "Type message...".to_string(),
            32,
            20.0,
            Color::WHITE,
            Color::DARK_GRAY,
        );

        let send_button = Button::new(
            "lobby_send_button",
            "SEND",
            Rect::new(0.785, 0.88, 80.0, 30.0),
            LayoutMetrics {
                anchor: AnchorPoint::TopLeft,
                positioning: LengthMode::Percent,
                sizing: LengthMode::Px,
            },
            Color::WHITE,
            Color::BLACK,
            Color::DARK_GRAY,
        );

        let lobby_maze_view = MazeView::new(
            "lobby_maze_view",
            self.maze.clone(),
            Rect::new(0.8, 0.05, 110.0, 110.0),
            LayoutMetrics {
                anchor: AnchorPoint::TopLeft,
                positioning: LengthMode::Percent,
                sizing: LengthMode::Px,
            },
            3.0,
        );

        let start_game_button = Button::new(
            "start_game_button",
            "START",
            Rect::new(0.9, 0.83, 80.0, 30.0),
            LayoutMetrics {
                anchor: AnchorPoint::TopLeft,
                positioning: LengthMode::Percent,
                sizing: LengthMode::Px,
            },
            Color::WHITE,
            Color::BLACK,
            Color::DARK_GRAY,
        );

        let leave_button = Button::new(
            "lobby_leave_button",
            "QUIT",
            Rect::new(0.9, 0.9, 80.0, 30.0),
            LayoutMetrics {
                anchor: AnchorPoint::TopLeft,
                positioning: LengthMode::Percent,
                sizing: LengthMode::Px,
            },
            Color::WHITE,
            Color::BLACK,
            Color::DARK_GRAY,
        );

        Layer {
            id: views::LOBBY.into(),
            z_index: 20,
            is_visible: false,
            is_modal: true,
            components: vec![
                Box::new(title),
                Box::new(username_label4),
                Box::new(lobby_addr_label),
                Box::new(lobby_maze_label),
                Box::new(user_list),
                Box::new(chat_log),
                Box::new(chat_input),
                Box::new(lobby_maze_view),
                Box::new(send_button),
                Box::new(start_game_button),
                Box::new(leave_button),
            ],
        }
    }

    fn handle_ui_events(&mut self, event: &UIEvent) -> Vec<ViewAction> {
        match event {
            UIEvent::TextChanged(id, text) if id == "lobby_chat_input" => {
                self.chat_input = text.clone();
                vec![]
            }

            UIEvent::TextSubmitted(id, _text) if id == "lobby_chat_input" => self.try_send_chat(),

            UIEvent::ButtonClicked(id) if id == "start_game_button" => {
                vec![ViewAction::SendStartGame]
            }

            UIEvent::ButtonClicked(id) if id == "lobby_send_button" => self.try_send_chat(),

            UIEvent::ButtonClicked(id) if id == "lobby_leave_button" => {
                vec![ViewAction::LeaveLobby]
            }

            _ => vec![],
        }
    }

    fn on_activate(&mut self, config: &Config) -> Vec<ComponentUpdate> {
        self.users.clear();
        self.messages.clear();
        self.chat_input.clear();

        vec![
            ComponentUpdate::SetText(
                "username_label4".into(),
                format!("as: {}", config.username.clone()),
            ),
            ComponentUpdate::SetText(
                "lobby_maze_settings".into(),
                format!(
                    "{:?} {:?} Maze | Max Players: {} | Win Score: {}",
                    self.maze.config.size,
                    self.maze.config.difficulty,
                    self.maze.config.max_players(),
                    self.target_score,
                ),
            ),
            ComponentUpdate::SetText(
"lobby_chat_log".into(),
"Controls:
- Use 'WASD' keys to move around
- Use 'SHIFT' key to run
- Use 'ESC' key to leave game
- Use the mouse to look around
- Use Left Mouse Click to shoot
- When hit, a player will turn invisible.".into(),
            ),
            ComponentUpdate::SetText("lobby_chat_input".into(), "".into()),
        ]
    }
}
