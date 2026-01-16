//! # Host Menu View
//!
//! View for hosting a new game. Allows users to configure game settings including
//! game name, target score, maze size, difficulty, and select custom mazes.

use super::{View, ViewAction};
use fps_config::Config;
use fps_levels::{
    config::{Difficulty, MazeConfig, MazeSize},
    generator::generate_maze,
    maze::Maze,
};
use fps_ui::{
    Color, ComponentUpdate, UIEvent,
    components::{Button, Label, MazeView, TextInput},
    geometry::Rect,
    layout::{AnchorPoint, LayoutMetrics, LengthMode},
    manager::Layer,
};
use crate::app::view_ids::views;

/// Host menu view implementation.
pub struct ViewHostMenu {
    /// Currently displayed maze (generated or custom).
    maze: Maze,
    
    /// Custom maze from configuration (if available).
    custom_maze: Maze,
    
    /// Current maze configuration (size and difficulty).
    maze_config: MazeConfig,

    /// Game name input value.
    game_name: String,
    
    /// Target score input value.
    target_score: String,
}

impl ViewHostMenu {
    /// Creates a new host menu view with default settings.
    pub fn new() -> Self {
        let maze_config = MazeConfig {
            size: MazeSize::Medium,
            difficulty: Difficulty::Normal,
        };

        let maze = generate_maze(&maze_config, "HOSTED_MAZE".to_string());
        let custom_maze = generate_maze(&maze_config, "CUSTOM_MAZE".to_string());

        Self {
            maze,
            maze_config,
            custom_maze,
            game_name: String::new(),
            target_score: String::new(),
        }
    }

    /// Regenerates the maze based on current configuration.
    ///
    /// If difficulty is Custom, uses the custom maze. Otherwise generates
    /// a new maze with the current configuration.
    fn regenerate_maze(&mut self) -> Vec<ViewAction> {
        if self.maze_config.difficulty == Difficulty::Custom {
            self.maze = self.custom_maze.clone();
        } else {
            self.maze = generate_maze(&self.maze_config, "HOSTED_MAZE".to_string());
        }

        vec![ViewAction::UpdateComponent(vec![
            ComponentUpdate::SetText("maze_view".into(), format!("[ Maze: {} ]", self.maze.name)),
            ComponentUpdate::SetText(
                "maze_setting_label".into(),
                format!(
                    "{:?} {:?} Maze. Max Players: {}",
                    self.maze.config.size,
                    self.maze.config.difficulty,
                    self.maze.config.max_players()
                ),
            ),
            ComponentUpdate::SetMaze("maze_view".into(), self.maze.clone()),
        ])]
    }
}

impl View for ViewHostMenu {
    fn id(&self) -> &str {
        views::HOST_MENU
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

        let username_label3 = Label::new(
            "username_label3".to_string(),
            format!("as: unknown"),
            24.0,
            Color::WHITE,
            Rect::new(0.5, 0.13, 400.0, 10.0),
            LayoutMetrics {
                anchor: AnchorPoint::TopLeft,
                positioning: LengthMode::Percent,
                sizing: LengthMode::Px,
            },
        );

        // Game name input
        let game_name_input = TextInput::new(
            "game_name_input".to_string(),
            Rect::new(0.05, 0.25, 360.0, 30.0),
            LayoutMetrics {
                anchor: AnchorPoint::TopLeft,
                positioning: LengthMode::Percent,
                sizing: LengthMode::Px,
            },
            "Enter game name...".to_string(),
            16,
            24.0,
            Color::WHITE,
            Color::DARK_GRAY,
        );

        let target_score_input = TextInput::new(
            "target_score_input".to_string(),
            Rect::new(0.05, 0.32, 360.0, 30.0),
            LayoutMetrics {
                anchor: AnchorPoint::TopLeft,
                positioning: LengthMode::Percent,
                sizing: LengthMode::Px,
            },
            "Enter target score...".to_string(),
            16,
            24.0,
            Color::WHITE,
            Color::DARK_GRAY,
        );

        let host_size_xsmall = Button::new(
            "host_size_xsmall",
            "X SMALL",
            Rect::new(0.05, 0.4, 120.0, 30.0),
            LayoutMetrics {
                anchor: AnchorPoint::TopLeft,
                positioning: LengthMode::Percent,
                sizing: LengthMode::Px,
            },
            Color::WHITE,
            Color::BLACK,
            Color::DARK_GRAY,
        );

        let host_size_small = Button::new(
            "host_size_small",
            "SMALL",
            Rect::new(0.05, 0.47, 120.0, 30.0),
            LayoutMetrics {
                anchor: AnchorPoint::TopLeft,
                positioning: LengthMode::Percent,
                sizing: LengthMode::Px,
            },
            Color::WHITE,
            Color::BLACK,
            Color::DARK_GRAY,
        );

        let host_size_medium = Button::new(
            "host_size_medium",
            "MEDIUM",
            Rect::new(0.05, 0.54, 120.0, 30.0),
            LayoutMetrics {
                anchor: AnchorPoint::TopLeft,
                positioning: LengthMode::Percent,
                sizing: LengthMode::Px,
            },
            Color::WHITE,
            Color::BLACK,
            Color::DARK_GRAY,
        );

        let host_size_big = Button::new(
            "host_size_big",
            "BIG",
            Rect::new(0.05, 0.61, 120.0, 30.0),
            LayoutMetrics {
                anchor: AnchorPoint::TopLeft,
                positioning: LengthMode::Percent,
                sizing: LengthMode::Px,
            },
            Color::WHITE,
            Color::BLACK,
            Color::DARK_GRAY,
        );

        let host_diff_easy = Button::new(
            "host_diff_easy",
            "EASY",
            Rect::new(0.22, 0.4, 180.0, 30.0),
            LayoutMetrics {
                anchor: AnchorPoint::TopLeft,
                positioning: LengthMode::Percent,
                sizing: LengthMode::Px,
            },
            Color::WHITE,
            Color::BLACK,
            Color::DARK_GRAY,
        );

        let host_diff_normal = Button::new(
            "host_diff_normal",
            "NORMAL",
            Rect::new(0.22, 0.47, 180.0, 30.0),
            LayoutMetrics {
                anchor: AnchorPoint::TopLeft,
                positioning: LengthMode::Percent,
                sizing: LengthMode::Px,
            },
            Color::WHITE,
            Color::BLACK,
            Color::DARK_GRAY,
        );

        let host_diff_hard = Button::new(
            "host_diff_hard",
            "HARD",
            Rect::new(0.22, 0.54, 180.0, 30.0),
            LayoutMetrics {
                anchor: AnchorPoint::TopLeft,
                positioning: LengthMode::Percent,
                sizing: LengthMode::Px,
            },
            Color::WHITE,
            Color::BLACK,
            Color::DARK_GRAY,
        );

        let host_custom_maze = Button::new(
            "host_custom_maze",
            "CUSTOM",
            Rect::new(0.22, 0.61, 180.0, 30.0),
            LayoutMetrics {
                anchor: AnchorPoint::TopLeft,
                positioning: LengthMode::Percent,
                sizing: LengthMode::Px,
            },
            Color::WHITE,
            Color::BLACK,
            Color::DARK_GRAY,
        );

        // Max players label
        let maze_setting_label = Label::new(
            "maze_setting_label".to_string(),
            format!(
                "{:?} {:?} Maze. Max Players: {}",
                self.maze_config.size,
                self.maze_config.difficulty,
                self.maze_config.max_players()
            ),
            24.0,
            Color::WHITE,
            Rect::new(0.1, 0.2, 300.0, 30.0),
            LayoutMetrics {
                anchor: AnchorPoint::TopLeft,
                positioning: LengthMode::Percent,
                sizing: LengthMode::Px,
            },
        );

        let maze_view = MazeView::new(
            "maze_view",
            self.maze.clone(),
            Rect::new(0.51, 0.25, 330.0, 330.0),
            LayoutMetrics {
                anchor: AnchorPoint::TopLeft,
                positioning: LengthMode::Percent,
                sizing: LengthMode::Px,
            },
            10.0,
        );

        let error_label = Label::new(
            "host_error_label".to_string(),
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

        // Host game button
        let host_game_button = Button::new(
            "host_game_button",
            "HOST",
            Rect::new(0.88, 0.83, 100.0, 30.0),
            LayoutMetrics {
                anchor: AnchorPoint::TopLeft,
                positioning: LengthMode::Percent,
                sizing: LengthMode::Px,
            },
            Color::WHITE,
            Color::BLACK,
            Color::DARK_GRAY,
        );

        // Back button (bottom-right)
        let back_button = Button::new(
            "back_host_button",
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
            id: views::HOST_MENU.into(),
            z_index: 10,
            is_visible: false,
            is_modal: false,
            components: vec![
                Box::new(title),
                Box::new(username_label3),
                Box::new(game_name_input),
                Box::new(target_score_input),
                Box::new(host_size_xsmall),
                Box::new(host_size_small),
                Box::new(host_size_medium),
                Box::new(host_size_big),
                Box::new(host_diff_easy),
                Box::new(host_diff_normal),
                Box::new(host_diff_hard),
                Box::new(host_custom_maze),
                Box::new(maze_view),
                Box::new(maze_setting_label),
                Box::new(error_label),
                Box::new(host_game_button),
                Box::new(back_button),
            ],
        }
    }

    fn handle_ui_events(&mut self, event: &UIEvent) -> Vec<ViewAction> {
        match event {
            UIEvent::ButtonClicked(id) => match id.as_str() {
                // Size
                "host_size_xsmall" => {
                    self.maze_config.size = MazeSize::XSmall;
                    self.regenerate_maze()
                }
                "host_size_small" => {
                    self.maze_config.size = MazeSize::Small;
                    self.regenerate_maze()
                }
                "host_size_medium" => {
                    self.maze_config.size = MazeSize::Medium;
                    self.regenerate_maze()
                }
                "host_size_big" => {
                    self.maze_config.size = MazeSize::Big;
                    self.regenerate_maze()
                }

                // Difficulty
                "host_diff_easy" => {
                    self.maze_config.difficulty = Difficulty::Easy;
                    self.regenerate_maze()
                }
                "host_diff_normal" => {
                    self.maze_config.difficulty = Difficulty::Normal;
                    self.regenerate_maze()
                }
                "host_diff_hard" => {
                    self.maze_config.difficulty = Difficulty::Hard;
                    self.regenerate_maze()
                }

                "host_custom_maze" => {
                    self.maze_config.difficulty = Difficulty::Custom;
                    self.regenerate_maze()
                }

                // Host
                "host_game_button" => {
                    vec![ViewAction::HostGame {
                        game_name: self.game_name.clone(),
                        maze: self.maze.clone(),
                        target_score: self.target_score.clone(),
                    }]
                }

                "back_host_button" => {
                    vec![ViewAction::SwitchTo(views::MAIN_MENU.to_string())]
                }

                _ => vec![],
            },

            UIEvent::TextChanged(id, text) => {
                match id.as_str() {
                    "game_name_input" => {
                        self.game_name = text.clone();
                    }
                    "target_score_input" => {
                        self.target_score = text.clone();
                    }
                    _ => {}
                }
                vec![]
            }

            _ => vec![],
        }
    }

    fn on_activate(&mut self, config: &Config) -> Vec<ComponentUpdate> {
        // Use the maze from config if available
        if let Some(maze) = &config.maze {
            self.custom_maze = maze.clone();
        }

        // reset properties
        self.game_name.clear();
        self.target_score.clear();

        vec![
            ComponentUpdate::SetText(
                "username_label3".into(),
                format!("as: {}", config.username.clone()),
            ),
            ComponentUpdate::SetText(
                "host_custom_maze".into(),
                format!("{}", self.custom_maze.name),
            ),
            ComponentUpdate::SetText("game_name_input".into(), "".to_string()),
            ComponentUpdate::SetText("target_score_input".into(), "".to_string()),
            ComponentUpdate::SetText(
                "maze_setting_label".into(),
                format!(
                    "{:?} {:?} Maze. Max Players: {}",
                    self.maze_config.size,
                    self.maze_config.difficulty,
                    self.maze_config.max_players()
                ),
            ),
            ComponentUpdate::SetText("maze_view".into(), format!("[ Maze: {} ]", self.maze.name)),
            ComponentUpdate::SetText("host_error_label".into(), String::new()),
        ]
    }
}
