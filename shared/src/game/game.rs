use serde::{Serialize, Deserialize};
use bincode::serde::{encode_to_vec, decode_from_slice};
use bincode::config;

use super::player::{Player, PlayerSnapshot};
use super::stage::Stage;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum GameState {
    InLobby,
    Starting,
    Running,
    Ending,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum GameMode {
    Single,
    Team,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Game {
    pub owner: String,
    pub state: GameState,
    pub mode: GameMode,
    pub stage: Stage,
    pub players: Vec<Player>,
    pub tick: u64,
}

impl Game {
    pub fn snapshot(&self) -> GameSnapShot {
        GameSnapShot {
            players: self.players.iter().map(|p| p.snapshot()).collect(),
            tick: self.tick,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GameSnapShot {
    pub players: Vec<PlayerSnapshot>,
    pub tick: u64,
}

impl GameSnapShot {
    pub fn serialize(&self) -> Vec<u8> {
        encode_to_vec(self, config::standard())
            .expect("Failed to serialize GameSnapShot")
    }

    pub fn deserialize(data: &[u8]) -> Self {
        decode_from_slice(data, config::standard())
            .expect("Failed to deserialize GameSnapShot")
            .0
    }
}