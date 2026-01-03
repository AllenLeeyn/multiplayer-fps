use super::{View, ViewAction};
use fps_config::Config;
use fps_levels::maze::Maze;

use fps_ui::{
    Color, ComponentUpdate, UIEvent,
    components::{Button, Label, TextBox, TextInput},
    geometry::Rect,
    layout::{AnchorPoint, LayoutMetrics, LengthMode},
    manager::Layer,
};

pub struct ViewLobby {
    maze: Maze,
    target_score: u32,

    users: Vec<String>,
    messages: Vec<String>,
    chat_input: String,
}

impl ViewLobby {
    pub fn new() -> Self {
        Self {
            maze: Maze::default(),
            target_score: 0,

            users: Vec::new(),
            messages: Vec::new(),
            chat_input: String::new(),
        }
    }

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
        "lobby"
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
            id: "lobby".into(),
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
                Box::new(send_button),
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
                    "{:?} {:?} Maze | Max Players: {} | Win Socre: {}",
                    self.maze.config.size,
                    self.maze.config.difficulty,
                    self.maze.config.max_players(),
                    self.target_score,
                ),
            ),
            ComponentUpdate::SetText("lobby_user_list".into(), "".into()),
            ComponentUpdate::SetText("lobby_chat_log".into(), "".into()),
            ComponentUpdate::SetText("lobby_chat_input".into(), "".into()),
        ]
    }
}
