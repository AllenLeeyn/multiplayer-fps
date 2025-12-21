use fps_ui::{
    Color, UIEvent,
    components::{Button, Label},
    geometry::Rect,
    layout::{AnchorPoint, LayoutMetrics, LengthMode},
    manager::Layer,
};

use super::{View, ViewAction};

pub struct ViewHostMenu;

impl ViewHostMenu {
    pub fn new() -> Self {
        Self
    }
}

impl View for ViewHostMenu {
    fn id(&self) -> &str {
        "host_menu"
    }

    fn layer(&self) -> Layer {
        // Title label (top-left)
        let title = Label::new(
            "host_title".to_string(),
            "Host Game".to_string(),
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
            "back_host_button",
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
            id: "host_menu".into(),
            z_index: 10,
            is_visible: false,
            is_modal: false,
            components: vec![Box::new(title), Box::new(back_button)],
        }
    }

    fn handle_ui_events(&mut self, event: &UIEvent) -> Vec<ViewAction> {
        match event {
            UIEvent::ButtonClicked(id) if id == "back_host_button" => {
                vec![ViewAction::SwitchTo("main_menu".to_string())]
            }
            _ => vec![],
        }
    }
}
