//! # Game Structures Module
//!
//! Defines core game state types including game states, player actions, and bullet structures.

use super::Pos;

/// Represents the current state of a game session.
///
/// Used to track game progression and manage state transitions on both
/// client and server sides.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GameState {
    /// Players are in the lobby, waiting for the game to start.
    Lobby,
    
    /// Game is transitioning from lobby to active gameplay.
    Starting,
    
    /// Game is actively running.
    InGame,
    
    /// Game has finished (someone won).
    Finished,
    
    /// Connection lost or disconnected.
    Disconnected,
}

use std::fmt;

impl fmt::Display for GameState {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let s = match self {
            GameState::Lobby => "lobby",
            GameState::Starting => "starting",
            GameState::InGame => "in_game",
            GameState::Finished => "finished",
            GameState::Disconnected => "disconnected",
        };
        write!(f, "{}", s)
    }
}

use std::str::FromStr;

impl FromStr for GameState {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "lobby" => Ok(GameState::Lobby),
            "starting" => Ok(GameState::Starting),
            "in_game" => Ok(GameState::InGame),
            "finished" => Ok(GameState::Finished),
            "disconnected" => Ok(GameState::Disconnected),
            _ => Err(()),
        }
    }
}

use winit::keyboard::KeyCode;

/// Player actions that can be performed in the game.
///
/// These actions are bound to keyboard keys and mouse buttons. Each action
/// is represented as a single byte for efficient network transmission.
#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum PlayerAction {
    /// Move forward (W key).
    MoveUp = 0,
    
    /// Move backward (S key).
    MoveDown = 1,
    
    /// Strafe left (A key).
    MoveLeft = 2,
    
    /// Strafe right (D key).
    MoveRight = 3,
    
    /// Run modifier (Left Shift key).
    Run = 4,
    
    /// Shoot (Left mouse button).
    Shoot = 5,
}

impl PlayerAction {
    /// Converts a keycode to a player action if the key is bound.
    ///
    /// # Arguments
    ///
    /// * `key` - The keyboard key code to convert
    ///
    /// # Returns
    ///
    /// The corresponding `PlayerAction` if the key is bound, `None` otherwise.
    pub fn from_keycode(key: KeyCode) -> Option<Self> {
        match key {
            KeyCode::KeyW => Some(PlayerAction::MoveUp),
            KeyCode::KeyS => Some(PlayerAction::MoveDown),
            KeyCode::KeyA => Some(PlayerAction::MoveLeft),
            KeyCode::KeyD => Some(PlayerAction::MoveRight),
            KeyCode::ShiftLeft => Some(PlayerAction::Run),
            _ => None,
        }
    }

    /// Converts the action to a single byte for network transmission.
    ///
    /// Uses the `#[repr(u8)]` representation to ensure consistent byte encoding.
    ///
    /// # Returns
    ///
    /// The byte representation of this action.
    pub fn to_byte(self) -> u8 {
        self as u8
    }

    /// Parses a byte back into a player action.
    ///
    /// # Arguments
    ///
    /// * `byte` - The byte to parse
    ///
    /// # Returns
    ///
    /// The corresponding `PlayerAction` if valid, `None` for invalid bytes.
    pub fn from_byte(byte: u8) -> Option<Self> {
        match byte {
            0 => Some(PlayerAction::MoveUp),
            1 => Some(PlayerAction::MoveDown),
            2 => Some(PlayerAction::MoveLeft),
            3 => Some(PlayerAction::MoveRight),
            4 => Some(PlayerAction::Run),
            5 => Some(PlayerAction::Shoot),
            _ => None,
        }
    }
}

/// Represents a bullet projectile in the game world.
///
/// Bullets are spawned when players shoot and move through the world
/// until they hit a wall or player.
#[derive(Debug, Clone)]
pub struct Bullet {
    /// ID of the player who shot this bullet.
    pub owner_id: String,
    
    /// Current position and direction of the bullet.
    pub pos: Pos,
}

/// Represents the status/result of a bullet's movement.
///
/// Used to track what happened to a bullet during physics update.
#[derive(Debug, Clone)]
pub enum BulletStatus {
    /// Bullet is still active and moving.
    Active,
    
    /// Bullet hit a wall and should be removed.
    HitWall,
    
    /// Bullet hit a player and should be removed.
    /// Contains the ID of the hit player.
    HitPlayer(String),
}