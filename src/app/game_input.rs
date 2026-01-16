use std::collections::HashSet;
use winit::event::{ElementState, MouseButton};
use winit::keyboard::{KeyCode, PhysicalKey};

use super::PlayerAction;
use fps_ui::WindowEvent;
use fps_net::message::GameInputPayload;

#[derive(Default, Debug)]
pub struct GameInputState {
    pub actions: HashSet<PlayerAction>,

    pub mouse_left_down: bool,
    pub mouse_dx: f32,

    pub left_shift_down: bool,
}

impl GameInputState {
    pub fn clear(&mut self) {
        self.actions.clear();
    }
    
    pub fn handle_game_input(
        &mut self,
        event: &WindowEvent,
    ) {
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

    pub fn handle_mouse_motion(&mut self, dx: f64) {
        self.mouse_dx += dx as f32;
    }

    pub fn snapshot(&mut self) -> GameInputPayload {
        let actions_bytes: HashSet<u8> = self.actions.iter().copied().map(|a| a.to_byte()).collect();

        let payload = GameInputPayload {
            actions: actions_bytes,
            is_running: self.left_shift_down,
            mouse_dx: self.mouse_dx,
        };

        self.mouse_dx = 0.0;

        payload
    }
}
