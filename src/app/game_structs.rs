use super::Pos;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GameState {
    Lobby,
    Starting,
    InGame,
    Finished,
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

/// Player actions
#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum PlayerAction {
    MoveUp = 0,
    MoveDown = 1,
    MoveLeft = 2,
    MoveRight = 3,
    Run = 4,
    Shoot =5,
}

impl PlayerAction {
    /// Convert a pressed key to a PlayerAction (if applicable)
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

    /// Convert the PlayerAction to a single byte for sending over the network
    pub fn to_byte(self) -> u8 {
        self as u8
    }

    /// Parse a byte back into a PlayerAction
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

#[derive(Debug, Clone)]
pub struct Bullet {
    pub owner_id: String,
    pub pos: Pos,
}

#[derive(Debug, Clone)]
pub enum BulletStatus {
    Active,
    HitWall,
    HitPlayer(String),
}