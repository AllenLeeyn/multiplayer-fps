use serde::{Serialize, Deserialize};
use glam::Vec3;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum PlayerType {
    Sphere,
    Cube,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[repr(u8)]
pub enum PlayerStatus {
    Alive = 0,
    Hit = 1,
    Dead = 2,
    Disconnected = 3,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[repr(u8)]
pub enum PlayerAction {
    Shoot = 0,
    Jump = 1,
    Ability = 2,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Player {
    pub id: u8,
    pub player_type: PlayerType,
    pub hp: u8,
    pub max_hp: u8,
    pub spd: f32,
    pub atk: u8,
    pub status: PlayerStatus,
    pub pos: Vec3,
    pub dir: Vec3,
    pub act: PlayerAction,
    pub team: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlayerSnapshot {
    pub id: u8,
    pub hp: u8,
    pub status: PlayerStatus,
    pub pos: Vec3,
    pub dir: Vec3,
    pub action: PlayerAction,
}

impl Player {
    pub fn snapshot(&self) -> PlayerSnapshot {
        PlayerSnapshot {
            id: self.id,
            hp: self.hp,
            status: self.status,
            pos: self.pos,
            dir: self.dir,
            action: self.act,
        }
    }
}
