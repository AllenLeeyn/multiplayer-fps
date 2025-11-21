use serde::{Serialize, Deserialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum StageDifficulty {
    Easy,
    Normal,
    Hard,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum StageType {
    User,
    Auto,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Stage {
    pub grid: Vec<Vec<u8>>,
    pub difficulty: StageDifficulty,
    pub stage_type: StageType,
    pub spawn_points: Vec<(u32, u32)>
}
