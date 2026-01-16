//! # Events Module
//!
//! Defines the event types used for communication between the UI system and the application.
//! Events flow from components to the application, while updates flow from the application to components.

use fps_levels::config::MazeSize;
use fps_levels::maze::Maze;
use std::collections::HashMap;
use std::fmt::Debug;
use super::Rect;

/// UI events sent from components to the application.
///
/// These events represent user interactions and component state changes that
/// the application needs to handle. The application receives these events from
/// `UIManager::process_input()` and should handle them appropriately.
#[derive(Debug, Clone)]
pub enum UIEvent {
    /// A button was clicked.
    ///
    /// Contains the button's component ID.
    ButtonClicked(String),
    
    /// A toggle button's state changed.
    ///
    /// Contains the button's component ID and new state.
    ButtonToggled(String, bool),
    
    /// Text in a text input changed.
    ///
    /// Contains the component ID and new text value.
    TextChanged(String, String),
    
    /// Text was submitted (e.g., Enter key pressed).
    ///
    /// Contains the component ID and submitted text.
    TextSubmitted(String, String),
    
    /// A numeric value changed (e.g., slider).
    ///
    /// Contains the component ID and new value.
    ValueChanged(String, f32),
    
    /// The user requested to exit the application.
    ExitRequested,
}

/// Component updates sent from the application to components.
///
/// These updates allow the application to modify component state (text, visibility,
/// position, etc.) without directly accessing component internals. Updates are
/// applied via `UIManager::apply_updates()`.
#[derive(Debug, Clone)]
pub enum ComponentUpdate {
    /// Sets the text content of a component.
    ///
    /// Parameters: `(component_id, new_text)`
    SetText(String, String),
    
    /// Sets the text content as a vector of lines.
    ///
    /// Parameters: `(component_id, lines)`
    SetTextVec(String, Vec<String>),
    
    /// Appends text to a component's existing text.
    ///
    /// Parameters: `(component_id, text_to_append)`
    AppendText(String, String),
    
    /// Appends multiple lines of text.
    ///
    /// Parameters: `(component_id, lines_to_append)`
    AppendTextVec(String, Vec<String>),
    
    /// Sets a numeric value (e.g., for sliders).
    ///
    /// Parameters: `(component_id, new_value)`
    SetValue(String, f32),
    
    /// Sets component visibility.
    ///
    /// Parameters: `(component_id, is_visible)`
    SetVisibility(String, bool),
    
    /// Sets layer visibility.
    ///
    /// Parameters: `(layer_id, is_visible)`
    SetLayerVisibility(String, bool),
    
    /// Sets component position.
    ///
    /// Parameters: `(component_id, new_x, new_y)`
    SetPosition(String, f64, f64),
    
    /// Resizes the UI canvas.
    ///
    /// Fields: `{ width, height }`
    Resize { width: f64, height: f64 },
    
    /// Sets hovered state.
    ///
    /// Parameters: `(component_id, is_hovered)`
    SetHovered(String, bool),
    
    /// Sets focus state.
    ///
    /// Parameters: `(component_id, is_focused)`
    SetFocus(String, bool),
    
    /// Sets maze size for maze-related components.
    ///
    /// Parameters: `(component_id, maze_size)`
    SetMazeSize(String, MazeSize),
    
    /// Sets maze data for maze-related components.
    ///
    /// Parameters: `(component_id, maze)`
    SetMaze(String, Maze),
    
    /// Sets minimap layout parameters.
    ///
    /// Parameters: `(component_id, (rect, cell_px, player_px))`
    SetMiniMapLayout(String, (Rect, f32, f32)),
    
    /// Sets minimap player position.
    ///
    /// Parameters: `(component_id, (x, y, angle))`
    SetMiniMapPlayer(String, (f32, f32, f32)),
    
    /// Sets game render data (players and bullets).
    ///
    /// Parameters: `(component_id, players_map, bullets_list)`
    /// - `players_map`: Map of player ID to `(x, y, angle, is_invincible)`
    /// - `bullets_list`: List of `(x, y, angle)` tuples
    SetGameRender(String, HashMap<String, (f32, f32, f32, bool)>, Vec<(f32, f32, f32)>),
    
    /// Sets game render camera (local player view).
    ///
    /// Parameters: `(component_id, (player_id, x, y, angle, is_invincible))`
    SetGameRenderCamera(String, (String, f32, f32, f32, bool)),
}

impl ComponentUpdate {
    /// Returns the target component ID if this update targets a component.
    ///
    /// # Returns
    ///
    /// The component ID if this update is component-specific, `None` if it's
    /// a layer or global update.
    pub fn get_target_id(&self) -> Option<&str> {
        match self {
            // These variants target a specific component ID
            ComponentUpdate::SetText(id, _)
            | ComponentUpdate::SetTextVec(id, _)
            | ComponentUpdate::AppendText(id, _)
            | ComponentUpdate::AppendTextVec(id, _)
            | ComponentUpdate::SetVisibility(id, _)
            | ComponentUpdate::SetValue(id, _)
            | ComponentUpdate::SetPosition(id, _, _)
            | ComponentUpdate::SetFocus(id, _)
            | ComponentUpdate::SetHovered(id, _)
            | ComponentUpdate::SetMazeSize(id, _)
            | ComponentUpdate::SetMiniMapPlayer(id, _)
            | ComponentUpdate::SetMiniMapLayout(id, _)
            | ComponentUpdate::SetGameRender(id, _, _)
            | ComponentUpdate::SetGameRenderCamera(id, _)
            | ComponentUpdate::SetMaze(id, _) => Some(id),

            // These variants target layers or are global, and should be handled separately
            ComponentUpdate::SetLayerVisibility(_, _) | ComponentUpdate::Resize { .. } => None,
        }
    }

    /// Returns the target layer ID if this update targets a layer.
    ///
    /// # Returns
    ///
    /// The layer ID if this update is layer-specific, `None` otherwise.
    pub fn get_layer_id(&self) -> Option<&str> {
        match self {
            ComponentUpdate::SetLayerVisibility(id, _) => Some(id),
            _ => None,
        }
    }
}

/// A snapshot of UI state changes to be applied atomically.
///
/// Allows batching multiple component updates together for efficient
/// state synchronization.
#[derive(Debug, Clone)]
pub struct UIStateSnapshot {
    /// The list of component updates to apply.
    pub updates: Vec<ComponentUpdate>,
}
