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

pub struct ViewHostMenu {
    maze: Maze,
    maze_config: MazeConfig,
}

impl ViewHostMenu {
    pub fn new() -> Self {
        let maze_config = MazeConfig {
            size: MazeSize::Medium,
            difficulty: Difficulty::Normal,
        };

        let maze = generate_maze(&maze_config, "HOSTED_MAZE".to_string());

        Self { maze, maze_config }
    }

    fn regenerate_maze(&mut self) -> Vec<ViewAction> {
        self.maze = generate_maze(&self.maze_config, "HOSTED_MAZE".to_string());

        vec![ViewAction::UpdateComponent(vec![
            ComponentUpdate::SetText("maze_view".into(), format!("[ Maze: {} ]", self.maze.name)),
            ComponentUpdate::SetText(
                "maze_setting_label".into(),
                format!(
                    "{:?} {:?} Maze. Max Players: {}",
                    self.maze_config.size,
                    self.maze_config.difficulty,
                    self.maze_config.max_players()
                ),
            ),
            ComponentUpdate::SetMaze("maze_view".into(), self.maze.clone()),
        ])]
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

        // Game name input
        let game_name_input = TextInput::new(
            "game_name_input".to_string(),
            Rect::new(0.05, 0.20, 360.0, 30.0),
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

        let host_size_xsmall = Button::new(
            "host_size_xsmall",
            "X SMALL",
            Rect::new(0.05, 0.28, 120.0, 30.0),
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
            Rect::new(0.05, 0.35, 120.0, 30.0),
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
            Rect::new(0.05, 0.42, 120.0, 30.0),
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
            Rect::new(0.05, 0.49, 120.0, 30.0),
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
            Rect::new(0.22, 0.28, 120.0, 30.0),
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
            Rect::new(0.22, 0.35, 120.0, 30.0),
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
            Rect::new(0.22, 0.42, 120.0, 30.0),
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
            16.0,
            Color::WHITE,
            Rect::new(0.05, 0.6, 300.0, 30.0),
            LayoutMetrics {
                anchor: AnchorPoint::TopLeft,
                positioning: LengthMode::Percent,
                sizing: LengthMode::Px,
            },
        );

        let maze_view = MazeView::new(
            "maze_view",
            self.maze.clone(),
            Rect::new(0.45, 0.28, 330.0, 330.0),
            LayoutMetrics {
                anchor: AnchorPoint::TopLeft,
                positioning: LengthMode::Percent,
                sizing: LengthMode::Px,
            },
            10.0,
        );

        // Host game button
        let host_game_button = Button::new(
            "host_game_button",
            "HOST",
            Rect::new(0.86, 0.83, 100.0, 30.0),
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
            components: vec![
                Box::new(title),
                Box::new(game_name_input),
                Box::new(host_size_xsmall),
                Box::new(host_size_small),
                Box::new(host_size_medium),
                Box::new(host_size_big),
                Box::new(host_diff_easy),
                Box::new(host_diff_normal),
                Box::new(host_diff_hard),
                Box::new(maze_view),
                Box::new(maze_setting_label),
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

                // Host
                /* "host_game_button" => vec![ViewAction::HostGame {
                    game_name_input: "game_name_input".to_string(),
                    maze: self.maze.clone(),
                }], */
                "back_host_button" => {
                    vec![ViewAction::SwitchTo("main_menu".to_string())]
                }

                _ => vec![],
            },

            _ => vec![],
        }
    }

    fn on_activate(&mut self, config: &Config) -> Vec<ComponentUpdate> {
        // Use the maze from config if available
        if let Some(maze) = &config.maze {
            self.maze = maze.clone();
        }

        vec![
            ComponentUpdate::SetText("game_name_input".into(), "".to_string()),
            ComponentUpdate::SetText("maze_view".into(), format!("[ Maze: {} ]", self.maze.name)),
            ComponentUpdate::SetText(
                "max_players_label".into(),
                format!("Max Players: {}", self.maze.config.max_players()),
            ),
        ]
    }
}
