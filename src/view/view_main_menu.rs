use fps_config::Config;
use fps_ui::{
    Color, ComponentUpdate, UIEvent,
    components::{Button, Label, TextInput},
    geometry::Rect,
    layout::{AnchorPoint, LayoutMetrics, LengthMode},
    manager::Layer,
};

use super::{View, ViewAction};

pub struct ViewMainMenu {
    pub username: String,
}

impl ViewMainMenu {
    pub fn new(username: String) -> Self {
        Self { username }
    }
}

impl View for ViewMainMenu {
    fn id(&self) -> &str {
        "main_menu"
    }

    fn layer(&self) -> Layer {
        let title = Label::new(
            "title".to_string(),
            "aMAZE".to_string(),
            180.0,
            Color::WHITE,
            Rect::new(0.11, 0.115, 1.0, 1.0),
            LayoutMetrics {
                anchor: AnchorPoint::TopLeft,
                positioning: LengthMode::Percent,
                sizing: LengthMode::Percent,
            },
        );
        let title2 = Label::new(
            "title".to_string(),
            "aMAZE".to_string(),
            180.0,
            Color::BLACK,
            Rect::new(0.115, 0.125, 1.0, 1.0),
            LayoutMetrics {
                anchor: AnchorPoint::TopLeft,
                positioning: LengthMode::Percent,
                sizing: LengthMode::Percent,
            },
        );

        let username_label = Label::new(
            "username_label".to_string(),
            format!("Current user: {}", self.username),
            24.0,
            Color::WHITE,
            Rect::new(0.11, 0.55, 400.0, 10.0),
            LayoutMetrics {
                anchor: AnchorPoint::TopLeft,
                positioning: LengthMode::Percent,
                sizing: LengthMode::Px,
            },
        );

        let input = TextInput::new(
            "username_input".to_string(),
            Rect::new(0.11, 0.6, 300.0, 30.0),
            LayoutMetrics {
                anchor: AnchorPoint::TopLeft,
                positioning: LengthMode::Percent,
                sizing: LengthMode::Px,
            },
            "Enter username....".to_string(),
            16,
            24.0,
            Color::WHITE,
            Color::DARK_GRAY,
        );

        let save_username = Button::new(
            "save_username_button",
            "SAVE",
            Rect::new(0.49, 0.6, 160.0, 30.0),
            LayoutMetrics {
                anchor: AnchorPoint::TopLeft,
                positioning: LengthMode::Percent,
                sizing: LengthMode::Px,
            },
            Color::WHITE,
            Color::BLACK,
            Color::DARK_GRAY,
        );
        let join = Button::new(
            "join_button",
            "JOIN GAME",
            Rect::new(0.11, 0.68, 200.0, 30.0),
            LayoutMetrics {
                anchor: AnchorPoint::TopLeft,
                positioning: LengthMode::Percent,
                sizing: LengthMode::Px,
            },
            Color::WHITE,
            Color::BLACK,
            Color::DARK_GRAY,
        );
        let host = Button::new(
            "host_button",
            "HOST GAME",
            Rect::new(0.11, 0.75, 200.0, 30.0),
            LayoutMetrics {
                anchor: AnchorPoint::TopLeft,
                positioning: LengthMode::Percent,
                sizing: LengthMode::Px,
            },
            Color::WHITE,
            Color::BLACK,
            Color::DARK_GRAY,
        );
        let level = Button::new(
            "level_button",
            "LEVEL EDITOR",
            Rect::new(0.11, 0.82, 200.0, 30.0),
            LayoutMetrics {
                anchor: AnchorPoint::TopLeft,
                positioning: LengthMode::Percent,
                sizing: LengthMode::Px,
            },
            Color::WHITE,
            Color::BLACK,
            Color::DARK_GRAY,
        );
        let quit = Button::new(
            "quit_button",
            "QUIT",
            Rect::new(0.11, 0.89, 200.0, 30.0),
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
            id: "main_menu".into(),
            z_index: 10,
            is_visible: true,
            is_modal: false,
            components: vec![
                Box::new(title2),
                Box::new(title),
                Box::new(username_label),
                Box::new(input),
                Box::new(save_username),
                Box::new(join),
                Box::new(host),
                Box::new(level),
                Box::new(quit),
            ],
        }
    }

    fn handle_ui_events(&mut self, event: &UIEvent) -> Vec<ViewAction> {
        match event {
            UIEvent::ButtonClicked(id) if id == "save_username_button" => {
                vec![ViewAction::SaveUsername]
            }

            UIEvent::ButtonClicked(id) if id == "join_button" => {
                vec![ViewAction::SwitchTo("join_menu".to_string())]
            }

            UIEvent::ButtonClicked(id) if id == "host_button" => {
                vec![ViewAction::SwitchTo("host_menu".to_string())]
            }

            UIEvent::ButtonClicked(id) if id == "level_button" => {
                vec![ViewAction::SwitchTo("level_menu".to_string())]
            }

            UIEvent::ButtonClicked(id) if id == "quit_button" => {
                vec![ViewAction::QuitApp]
            }

            UIEvent::TextSubmitted(id, _value) if id == "username_input" => {
                vec![ViewAction::SaveUsername]
            }

            _ => vec![],
        }
    }

    fn on_activate(&mut self, config: &Config) -> Vec<ComponentUpdate> {
        self.username = config.username.clone(); // sync internal state

        vec![
            ComponentUpdate::SetText(
                "username_label".into(),
                format!("Current user: {}", self.username),
            ),
            ComponentUpdate::SetText("username_input".into(), String::new()), // clear input
        ]
    }
}
