use fps_ui::{
    Color, ComponentUpdate, UIEvent,
    components::{Button, Label, TextInput},
    geometry::Rect,
    layout::{AnchorPoint, LayoutMetrics, LengthMode},
    manager::Layer,
};

use super::{View, ViewAction};
use fps_config::Config;

pub struct ViewJoinMenu {
    server_addr: String,
}

impl ViewJoinMenu {
    pub fn new() -> Self {
        Self {
            server_addr: String::new(),
        }
    }
}

impl View for ViewJoinMenu {
    fn id(&self) -> &str {
        "join_menu"
    }

    fn layer(&self) -> Layer {
        // Title label (top-left)
        let title = Label::new(
            "join_title".to_string(),
            "Join Game".to_string(),
            64.0,
            Color::WHITE,
            Rect::new(0.05, 0.08, 1.0, 1.0),
            LayoutMetrics {
                anchor: AnchorPoint::TopLeft,
                positioning: LengthMode::Percent,
                sizing: LengthMode::Percent,
            },
        );

        let username_label2 = Label::new(
            "username_label2".to_string(),
            format!("as: unknown"),
            24.0,
            Color::WHITE,
            Rect::new(0.05, 0.2, 400.0, 10.0),
            LayoutMetrics {
                anchor: AnchorPoint::TopLeft,
                positioning: LengthMode::Percent,
                sizing: LengthMode::Px,
            },
        );

        let address_input = TextInput::new(
            "address_input".to_string(),
            Rect::new(0.05, 0.25, 360.0, 30.0),
            LayoutMetrics {
                anchor: AnchorPoint::TopLeft,
                positioning: LengthMode::Percent,
                sizing: LengthMode::Px,
            },
            "Enter server address....".to_string(),
            18,
            24.0,
            Color::WHITE,
            Color::DARK_GRAY,
        );

        let connect_button = Button::new(
            "connect_button",
            "CONNECT",
            Rect::new(0.51, 0.25, 160.0, 30.0),
            LayoutMetrics {
                anchor: AnchorPoint::TopLeft,
                positioning: LengthMode::Percent,
                sizing: LengthMode::Px,
            },
            Color::WHITE,
            Color::BLACK,
            Color::DARK_GRAY,
        );

        let error_label = Label::new(
            "join_error_label".to_string(),
            "".to_string(), // initially empty
            16.0,           // font size
            Color::YELLOW,
            Rect::new(0.05, 0.83, 400.0, 30.0),
            LayoutMetrics {
                anchor: AnchorPoint::TopLeft,
                positioning: LengthMode::Percent,
                sizing: LengthMode::Px,
            },
        );

        // Back button (bottom-right)
        let back_button = Button::new(
            "back_join_button",
            "BACK",
            Rect::new(0.88, 0.90, 100.0, 30.0),
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
            id: "join_menu".into(),
            z_index: 10,
            is_visible: false, // start hidden, shown via SwitchTo
            is_modal: false,
            components: vec![
                Box::new(title),
                Box::new(username_label2),
                Box::new(address_input),
                Box::new(connect_button),
                Box::new(error_label),
                Box::new(back_button),
            ],
        }
    }

    fn handle_ui_events(&mut self, event: &UIEvent) -> Vec<ViewAction> {
        match event {
            UIEvent::ButtonClicked(id) => match id.as_str() {
                "connect_button" => {
                    vec![ViewAction::JoinGame(self.server_addr.to_string())]
                }
                "back_join_button" => {
                    vec![ViewAction::SwitchTo("main_menu".to_string())]
                }

                _ => vec![],
            },

            UIEvent::TextChanged(id, text) => {
                match id.as_str() {
                    "address_input" => {
                        self.server_addr = text.clone();
                    }
                    _ => {}
                }
                vec![]
            }

            _ => vec![],
        }
    }

    fn on_activate(&mut self, config: &Config) -> Vec<ComponentUpdate> {
        vec![
            ComponentUpdate::SetText(
                "username_label2".into(),
                format!("as: {}", config.username.clone()),
            ),
            ComponentUpdate::SetText("join_error_label".into(), String::new()),
        ]
    }
}
