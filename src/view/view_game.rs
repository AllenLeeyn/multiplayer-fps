//! # Game View
//!
//! The in-game view displayed during active gameplay. Contains the 3D game renderer,
//! minimap, leaderboard, and score display.

use super::super::{LOGICAL_HEIGHT, LOGICAL_WIDTH};
use super::{View, ViewAction};
use fps_config::Config;
use fps_levels::maze::Maze;

use fps_ui::{
    Color, ComponentUpdate, UIEvent,
    components::{GameRender, Label, TextBox, MiniMap},
    geometry::Rect,
    layout::{AnchorPoint, LayoutMetrics, LengthMode},
    manager::Layer,
};
use crate::app::view_ids::views;

/// Game view implementation.
pub struct ViewGame {
    /// Bounds of the game render area.
    pub bounds: Rect,
    
    /// Current maze for the game.
    pub maze: Maze,
    
    /// Whether the left mouse button is currently pressed.
    pub is_left_mouse_btn_pressed: bool,
}

impl ViewGame {
    /// Creates a new game view.
    pub fn new() -> Self {
        Self {
            maze: Maze::default(),
            bounds: Rect::new(0.0, 0.0, LOGICAL_WIDTH as f64, LOGICAL_HEIGHT as f64),
            is_left_mouse_btn_pressed: false,
        }
    }
}

impl View for ViewGame {
    fn id(&self) -> &str {
        views::GAME
    }

    fn layer(&self) -> Layer {
        let game_render: GameRender = GameRender::new("game_render".to_string(), self.bounds);

        let game_score_label: Label = Label::new(
            "game_score_label".to_string(),
            "-".to_string(),
            16.0,
            Color::YELLOW,
            Rect::new(0.008, 0.04, 200.0, 16.0),
            LayoutMetrics {
                anchor: AnchorPoint::TopLeft,
                positioning: LengthMode::Percent,
                sizing: LengthMode::Px,
            },
        );

        let game_leaderboard = TextBox::new(
            "game_leaderboard".to_string(),
            Rect::new(0.0, 0.03, 200.0, 72.0),
            LayoutMetrics {
                anchor: AnchorPoint::TopLeft,
                positioning: LengthMode::Percent,
                sizing: LengthMode::Px,
            },
            16.0,
            Color::WHITE,
            Color::BLACK,
        );

        let game_mini_map = MiniMap::new(
            "game_mini_map".to_string(),
            self.maze.clone(),
            Rect::new(0.0, 0.1, 100.0, 100.0),
            LayoutMetrics {
                anchor: AnchorPoint::TopLeft,
                positioning: LengthMode::Px,
                sizing: LengthMode::Px,
            },
            100.0,
            50.0,
        );
        // mini-map (maze)
        // world

        Layer {
            id: self.id().into(),
            z_index: 20,
            is_visible: false,
            is_modal: true,
            components: vec![
                Box::new(game_render),
                Box::new(game_leaderboard),
                Box::new(game_score_label),
                Box::new(game_mini_map)
            ],
        }
    }

    fn handle_ui_events(&mut self, event: &UIEvent) -> Vec<ViewAction> {
        match event {
            UIEvent::ButtonClicked(id) if id == "game_render" => {
                vec![ViewAction::LeaveLobby]
            }

            _ => vec![],
        }
    }

    fn on_activate(&mut self, _config: &Config) -> Vec<ComponentUpdate> {
        Vec::new()
    }
}
