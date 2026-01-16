//! # UI Components Module
//!
//! Provides a collection of reusable UI components that implement the [`Component`] trait.
//! Each component handles its own rendering, input processing, and state management.
//!
//! ## Available Components
//!
//! ### Basic Components
//!
//! - **[`Button`]**: Interactive clickable button with text and hover effects
//! - **[`Label`]**: Static text display component
//! - **[`Panel`]**: Rectangular background/container with solid color or texture
//! - **[`FpsComponent`]**: Real-time FPS counter display
//!
//! ### Text Components
//!
//! - **[`TextInput`]**: Single-line editable text field with cursor
//! - **[`TextBox`]**: Multi-line scrollable text display
//!
//! ### Game-Specific Components
//!
//! - **[`GameRender`]**: 3D raycasted game world renderer
//! - **[`MiniMap`]**: Top-down minimap with player position
//! - **[`MazeView`]**: Read-only maze visualization
//! - **[`MazeEditor`]**: Interactive maze editor with click-to-edit
//!
//! ## Usage Pattern
//!
//! ```rust,no_run
//! use fps_ui::components::{Button, Label};
//! use fps_ui::{Color, Rect, layout::LayoutMetrics};
//!
//! // Create a button
//! let button = Button::new(
//!     "my_button",
//!     "Click Me",
//!     Rect::new(100.0, 100.0, 200.0, 50.0),
//!     LayoutMetrics::default(),
//!     Color::WHITE,
//!     Color::BLUE,
//!     Color::CYAN,
//! );
//!
//! // Add to a layer
//! let layer = Layer {
//!     id: "main".to_string(),
//!     components: vec![Box::new(button)],
//!     // ...
//! };
//! ```

pub mod button;
pub mod component;
pub mod fps_counter;
pub mod game_render;
pub mod label;
pub mod maze_editor;
pub mod maze_viewer;
pub mod panel;
pub mod text_box;
pub mod text_input;
pub mod mini_map;

pub use button::Button;
pub use component::Component;
pub use fps_counter::FpsComponent;
pub use game_render::GameRender;
pub use label::Label;
pub use maze_editor::MazeEditor;
pub use maze_viewer::MazeView;
pub use panel::Panel;
pub use text_box::TextBox;
pub use text_input::TextInput;
pub use mini_map::MiniMap;
