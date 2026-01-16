//! # Game Input Module
//!
//! Manages player input state for game controls. Tracks keyboard and mouse inputs,
//! converts them to player actions, and creates network payloads for sending to the server.

use std::collections::HashSet;
use winit::event::{ElementState, MouseButton};
use winit::keyboard::{KeyCode, PhysicalKey};

use super::PlayerAction;
use fps_ui::WindowEvent;
use fps_net::message::GameInputPayload;

/// Tracks the current state of player input.
///
/// Accumulates input events over a frame and provides methods to create
/// network payloads for sending to the server. Handles both keyboard and mouse input.
#[derive(Default, Debug)]
pub struct GameInputState {
    /// Set of currently active player actions.
    pub actions: HashSet<PlayerAction>,

    /// Whether the left mouse button is currently pressed.
    pub mouse_left_down: bool,
    
    /// Accumulated mouse delta X (horizontal movement) since last snapshot.
    pub mouse_dx: f32,

    /// Whether the left shift key is currently pressed (for running).
    pub left_shift_down: bool,
}

impl GameInputState {
    /// Clears all tracked actions.
    ///
    /// Useful for resetting input state when switching contexts or
    /// when input should be disabled.
    pub fn clear(&mut self) {
        self.actions.clear();
    }

    /// Processes a window event and updates input state.
    ///
    /// Handles keyboard and mouse button events, converting them to
    /// player actions. Should be called for each window event during
    /// the input processing phase.
    ///
    /// # Arguments
    ///
    /// * `event` - The window event to process
    pub fn handle_game_input(&mut self, event: &WindowEvent) {
        match event {
            WindowEvent::KeyboardInput { event, .. } => {
                if let PhysicalKey::Code(code) = event.physical_key {
                    if let Some(action) = PlayerAction::from_keycode(code) {
                        match event.state {
                            ElementState::Pressed => {
                                self.actions.insert(action);
                                if code == KeyCode::ShiftLeft {
                                    self.left_shift_down = true;
                                }
                            }
                            ElementState::Released => {
                                self.actions.remove(&action);
                                if code == KeyCode::ShiftLeft {
                                    self.left_shift_down = false;
                                }
                            }
                        }
                    }
                }
            }

            WindowEvent::MouseInput { state, button, .. } => {
                if *button == MouseButton::Left {
                    match state {
                        ElementState::Pressed => {
                            if !self.mouse_left_down {
                                self.actions.insert(PlayerAction::Shoot);
                            }
                            self.mouse_left_down = true;
                        }
                        ElementState::Released => {
                            self.actions.remove(&PlayerAction::Shoot);
                            self.mouse_left_down = false;
                        }
                    }
                }
            }

            _ => {}
        }
    }

    /// Accumulates mouse motion for rotation.
    ///
    /// Mouse delta X is accumulated and reset when creating a snapshot.
    /// This allows smooth rotation tracking even with frame-rate variations.
    ///
    /// # Arguments
    ///
    /// * `dx` - Mouse delta X (horizontal movement)
    pub fn handle_mouse_motion(&mut self, dx: f64) {
        self.mouse_dx += dx as f32;
    }

    /// Creates a network payload from the current input state.
    ///
    /// Converts the accumulated input state into a `GameInputPayload` that
    /// can be serialized and sent to the server. Resets mouse delta X after
    /// capturing.
    ///
    /// # Returns
    ///
    /// A `GameInputPayload` containing all current input state.
    pub fn snapshot(&mut self) -> GameInputPayload {
        let actions_bytes: HashSet<u8> = self.actions.iter().copied().map(|a| a.to_byte()).collect();

        let payload = GameInputPayload {
            actions: actions_bytes,
            is_running: self.left_shift_down,
            mouse_dx: self.mouse_dx,
        };

        // Reset accumulated mouse delta after capturing
        self.mouse_dx = 0.0;

        payload
    }
}
