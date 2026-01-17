//! # Join Menu View
//!
//! View for joining an existing game server. Allows users to enter a server
//! address and connect to a game.

use std::collections::HashMap;

use fps_ui::{
    Color, Component,ComponentUpdate, UIEvent,
    components::{Button, Label, TextInput},
    geometry::Rect,
    layout::{AnchorPoint, LayoutMetrics, LengthMode},
    manager::Layer,
};

use super::{View, ViewAction};
use fps_config::Config;
use crate::app::view_ids::views;

/// Join menu view implementation.
pub struct ViewJoinMenu {
    /// Server address input value.
    server_addr: String,
    /// Server alias input value.
    server_alias: String,
    /// Saved servers.
    servers: HashMap<String, String>,
}

impl ViewJoinMenu {
    /// Creates a new join menu view.
    pub fn new() -> Self {
        Self {
            server_addr: String::new(),
            server_alias: String::new(),
            servers: HashMap::new(),
        }
    }
}

impl View for ViewJoinMenu {
    fn id(&self) -> &str {
        views::JOIN_MENU
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

        let address_alias_input = TextInput::new(
            "address_alias_input".to_string(),
            Rect::new(0.05, 0.25, 180.0, 30.0),
            LayoutMetrics {
                anchor: AnchorPoint::TopLeft,
                positioning: LengthMode::Percent,
                sizing: LengthMode::Px,
            },
            "alias".to_string(),
            12,
            24.0,
            Color::WHITE,
            Color::DARK_GRAY,
        );

        let address_input = TextInput::new(
            "address_input".to_string(),
            Rect::new(0.28, 0.25, 280.0, 30.0),
            LayoutMetrics {
                anchor: AnchorPoint::TopLeft,
                positioning: LengthMode::Percent,
                sizing: LengthMode::Px,
            },
            "Enter server address".to_string(),
            18,
            24.0,
            Color::WHITE,
            Color::DARK_GRAY,
        );

        let connect_button = Button::new(
            "connect_button",
            "CONNECT",
            Rect::new(0.64, 0.25, 160.0, 30.0),
            LayoutMetrics {
                anchor: AnchorPoint::TopLeft,
                positioning: LengthMode::Percent,
                sizing: LengthMode::Px,
            },
            Color::WHITE,
            Color::BLACK,
            Color::DARK_GRAY,
        );

        let save_button = Button::new(
            "save_button",
            "SAVE",
            Rect::new(0.85, 0.25, 80.0, 30.0),
            LayoutMetrics {
                anchor: AnchorPoint::TopLeft,
                positioning: LengthMode::Percent,
                sizing: LengthMode::Px,
            },
            Color::WHITE,
            Color::BLACK,
            Color::DARK_GRAY,
        );

        // Create saved server buttons using a loop (up to 5)
        let mut saved_server_buttons = Vec::new();
        for i in 0..5 {
            let y_pos = 0.33 + (i as f64 * 0.07); // 0.33, 0.40, 0.47, 0.54, 0.61
            let button = Button::new(
                &format!("saved_server_{}", i),
                "",
                Rect::new(0.05, y_pos, 300.0, 30.0),
                LayoutMetrics {
                    anchor: AnchorPoint::TopLeft,
                    positioning: LengthMode::Percent,
                    sizing: LengthMode::Px,
                },
                Color::WHITE,
                Color::BLACK,
                Color::DARK_GRAY,
            );
            saved_server_buttons.push(Box::new(button) as Box<dyn Component>);
        }

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

        let mut components: Vec<Box<dyn Component>> = vec![
            Box::new(title),
            Box::new(username_label2),
            Box::new(address_input),
            Box::new(address_alias_input),
            Box::new(connect_button),
            Box::new(save_button),
            Box::new(error_label),
            Box::new(back_button),
        ];
        
        for button in saved_server_buttons {
            components.push(button);
        }

        Layer {
            id: views::JOIN_MENU.into(),
            z_index: 10,
            is_visible: false, // start hidden, shown via SwitchTo
            is_modal: false,
            components,
        }
    }

    fn handle_ui_events(&mut self, event: &UIEvent) -> Vec<ViewAction> {
        match event {
            UIEvent::ButtonClicked(id) => match id.as_str() {
                "connect_button" => {
                    vec![ViewAction::JoinGame(self.server_addr.to_string())]
                }
                "save_button" => {
                    vec![ViewAction::SaveServer(self.server_addr.to_string(), self.server_alias.to_string())]
                }
                "back_join_button" => {
                    vec![ViewAction::SwitchTo(views::MAIN_MENU.to_string())]
                }
                id if id.starts_with("saved_server_") => {
                    // Extract index from button ID (e.g., "saved_server_0" -> 0)
                    if let Some(index_str) = id.strip_prefix("saved_server_") {
                        if let Ok(index) = index_str.parse::<usize>() {
                            let servers_vec: Vec<(&String, &String)> = self.servers.iter().take(5).collect();
                            if index < servers_vec.len() {
                                let (alias, address) = servers_vec[index];
                                self.server_addr = address.clone();
                                self.server_alias = alias.clone();
                                // Return action to update the input fields
                                vec![
                                    ViewAction::UpdateComponent(vec![
                                        ComponentUpdate::SetText("address_input".into(), address.clone()),
                                        ComponentUpdate::SetText("address_alias_input".into(), alias.clone()),
                                    ])
                                ]
                            } else {
                                vec![]
                            }
                        } else {
                            vec![]
                        }
                    } else {
                        vec![]
                    }
                }
                _ => vec![],
            },

            UIEvent::TextChanged(id, text) => {
                match id.as_str() {
                    "address_input" => {
                        self.server_addr = text.clone();
                    }
                    "address_alias_input" => {
                        self.server_alias = text.clone();
                    }
                    _ => {}
                }
                vec![]
            }

            _ => vec![],
        }
    }

    fn on_activate(&mut self, config: &Config) -> Vec<ComponentUpdate> {
        // Load saved servers from config
        self.servers = config.saved_servers.clone();
        
        let mut updates = vec![
            ComponentUpdate::SetText(
                "username_label2".into(),
                format!("as: {}", config.username.clone()),
            ),
            ComponentUpdate::SetText("join_error_label".into(), String::new()),
        ];
        
        // Update saved server buttons (up to 5)
        let servers_vec: Vec<(&String, &String)> = self.servers.iter().take(5).collect();
        for (index, (alias, _address)) in servers_vec.iter().enumerate() {
            let button_id = format!("saved_server_{}", index);
            let button_text = format!("{}", alias);
            updates.push(ComponentUpdate::SetText(button_id, button_text));
        }
        
        // Hide unused buttons
        for index in servers_vec.len()..5 {
            let button_id = format!("saved_server_{}", index);
            updates.push(ComponentUpdate::SetText(button_id, String::new()));
        }
        
        updates
    }
}
