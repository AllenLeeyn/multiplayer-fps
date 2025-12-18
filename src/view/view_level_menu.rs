use fps_ui::{
    ComponentUpdate,
    Color, UIEvent,
    components::{Button, Label, TextInput},
    geometry::Rect,
    layout::{AnchorPoint, LayoutMetrics, LengthMode},
    manager::Layer,
};

use fps_config::Config;
use super::{View, ViewAction};

pub struct ViewLevelMenu;

impl ViewLevelMenu {
    pub fn new() -> Self {
        Self
    }
}

impl View for ViewLevelMenu {
    fn id(&self) -> &str {
        "level_menu"
    }

    fn layer(&self) -> Layer {
        // Title label (top-left)
        let title = Label::new(
            "level_title".to_string(),
            "Level editor".to_string(),
            64.0,
            Color::WHITE,
            Rect::new(0.05, 0.08, 1.0, 1.0),
            LayoutMetrics {
                anchor: AnchorPoint::TopLeft,
                positioning: LengthMode::Percent,
                sizing: LengthMode::Percent,
            },
        );


        // Back button (bottom-right)
        let back_button = Button::new(
            "back_button",
            "BACK",
            Rect::new(0.86, 0.90, 100.0, 30.0),
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
            id: "level_menu".into(),
            z_index: 10,
            is_visible: false,
            is_modal: false,
            components: vec![
                Box::new(title),
                Box::new(back_button),
            ],
        }
    }

    fn handle_ui_events(&mut self, event: &UIEvent) -> Vec<ViewAction> {
        match event {
            UIEvent::ButtonClicked(id) if id == "back_button" => {
                vec![ViewAction::SwitchTo("main_menu".to_string())]
            }
            _ => vec![],
        }
    }

}
