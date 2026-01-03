use fps_ui::{
    Color, ComponentUpdate, UIEvent,
    components::{Button, Label, MazeEditor, TextInput},
    geometry::Rect,
    layout::{AnchorPoint, LayoutMetrics, LengthMode},
    manager::Layer,
};

use super::{View, ViewAction};
use fps_config::Config;
use fps_levels::config::MazeSize;
use fps_levels::maze::Maze;

pub struct ViewLevelMenu {
    maze_name: String,
}

impl ViewLevelMenu {
    pub fn new() -> Self {
        Self {
            maze_name: String::new(),
        }
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

        let maze_name_input = TextInput::new(
            "maze_name_input".to_string(),
            Rect::new(0.05, 0.2, 360.0, 30.0),
            LayoutMetrics {
                anchor: AnchorPoint::TopLeft,
                positioning: LengthMode::Percent,
                sizing: LengthMode::Px,
            },
            "Enter maze name....".to_string(),
            16,
            24.0,
            Color::WHITE,
            Color::DARK_GRAY,
        );

        let xsmall_button = Button::new(
            "maze_size_xsmall",
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

        let small_button = Button::new(
            "maze_size_small",
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

        let medium_button = Button::new(
            "maze_size_medium",
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

        let big_button = Button::new(
            "maze_size_big",
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

        let maze_editor = MazeEditor::new(
            "maze_editor",
            MazeSize::Medium, // default size
            Rect::new(180.0, 125.0, 330.0, 330.0),
            LayoutMetrics {
                anchor: AnchorPoint::TopLeft,
                positioning: LengthMode::Px,
                sizing: LengthMode::Px,
            },
            12.0, // pixels per cell
        );

        let save_button = Button::new(
            "save_maze_button",
            "SAVE",
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
            "back_level_button",
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
            id: "level_menu".into(),
            z_index: 10,
            is_visible: false,
            is_modal: false,
            components: vec![
                Box::new(title),
                Box::new(maze_name_input),
                Box::new(xsmall_button),
                Box::new(small_button),
                Box::new(medium_button),
                Box::new(big_button),
                Box::new(maze_editor),
                Box::new(save_button),
                Box::new(back_button),
            ],
        }
    }

    fn handle_ui_events(&mut self, event: &UIEvent) -> Vec<ViewAction> {
        match event {
            UIEvent::ButtonClicked(id) => match id.as_str() {
                "maze_size_xsmall" => vec![ViewAction::UpdateComponent(vec![
                    ComponentUpdate::SetMazeSize("maze_editor".to_string(), MazeSize::XSmall),
                ])],

                "maze_size_small" => vec![ViewAction::UpdateComponent(vec![
                    ComponentUpdate::SetMazeSize("maze_editor".to_string(), MazeSize::Small),
                ])],

                "maze_size_medium" => vec![ViewAction::UpdateComponent(vec![
                    ComponentUpdate::SetMazeSize("maze_editor".to_string(), MazeSize::Medium),
                ])],

                "maze_size_big" => vec![ViewAction::UpdateComponent(vec![
                    ComponentUpdate::SetMazeSize("maze_editor".to_string(), MazeSize::Big),
                ])],

                "save_maze_button" => vec![ViewAction::SaveMaze(
                    self.maze_name.clone(),
                    "maze_editor".to_string(),
                )],

                "back_level_button" => vec![ViewAction::SwitchTo("main_menu".to_string())],

                _ => vec![],
            },

            UIEvent::TextChanged(id, value) if id == "maze_name_input" => {
                self.maze_name = value.clone();
                vec![]
            }

            _ => vec![],
        }
    }

    fn on_activate(&mut self, config: &Config) -> Vec<ComponentUpdate> {
        let maze = Maze::new("UNKOWN_MAZE".to_string(), 15, 15);
        self.maze_name = maze.name.clone();

        vec![
            ComponentUpdate::SetText("save_maze_button".into(), format!("SAVE")),
            ComponentUpdate::SetText(
                "maze_name_input".into(),
                config.maze.as_ref().unwrap_or(&maze).name.clone(),
            ),
            ComponentUpdate::SetMaze(
                "maze_editor".into(),
                config.maze.as_ref().unwrap_or(&maze).clone(),
            ),
        ]
    }
}
