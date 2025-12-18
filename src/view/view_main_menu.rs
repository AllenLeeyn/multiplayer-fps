use fps_ui::{
    Color, UIEvent,
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
            Rect::new(0.11, 0.6, 400.0, 30.0),
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

        let connect = Button::new(
            "connect_button",
            "CONNECT",
            Rect::new(0.11, 0.7, 160.0, 40.0),
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
                Box::new(connect),
            ],
        }
    }

    fn handle_ui_events(&mut self, event: &UIEvent) -> Vec<ViewAction> {
        match event {
            UIEvent::ButtonClicked(id) if id == "connect_button" => {
                vec![ViewAction::SaveUsername]
            }

            UIEvent::TextSubmitted(id, _value) if id == "username_input" => {
                vec![ViewAction::SaveUsername]
            }

            _ => vec![],
        }
    }
}
