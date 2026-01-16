//! # Configuration Module
//!
//! Defines configuration types for maze generation including size, difficulty, and related settings.

use serde::{Deserialize, Serialize};

/// Represents the number of rooms per side in the maze.
///
/// Each room is 3x3 cells, so the total grid size is `room_count() * 3`.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, Default)]
pub enum MazeSize {
    XSmall, // 3x3 rooms
    #[default]
    Small, // 5x5 rooms
    Medium, // 7x7 rooms
    Big,    // 9x9 rooms
}

impl MazeSize {
    /// Returns the number of rooms per side
    pub fn room_count(&self) -> usize {
        match self {
            MazeSize::XSmall => 3,
            MazeSize::Small => 5,
            MazeSize::Medium => 7,
            MazeSize::Big => 9,
        }
    }

    /// Returns the total grid size in cells (each room is 3x3)
    pub fn grid_size(&self) -> usize {
        self.room_count() * 3
    }
}

/// Difficulty level affecting maze generation parameters.
///
/// Primarily controls the loop chance during maze generation, which affects
/// how many alternative paths exist in the maze.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Default)]
pub enum Difficulty {
    Easy,
    #[default]
    Normal,
    Hard,
    Custom,
}

impl Difficulty {
    pub fn loop_chance(&self) -> f32 {
        match self {
            Difficulty::Easy => 0.25,
            Difficulty::Normal => 0.15,
            Difficulty::Hard => 0.01,
            Difficulty::Custom => 0.1,
        }
    }
}

/// Complete configuration for maze generation.
///
/// Combines size and difficulty settings to control the procedural generation process.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, Default)]
pub struct MazeConfig {
    pub size: MazeSize,
    pub difficulty: Difficulty,
}

impl MazeConfig {
    pub fn new() -> Self {
        Self {
            size: MazeSize::Small,
            difficulty: Difficulty::Custom,
        }
    }
    pub fn room_count(&self) -> usize {
        self.size.room_count()
    }

    pub fn grid_size(&self) -> usize {
        self.size.grid_size()
    }

    pub fn max_players(&self) -> usize {
        match self.size {
            MazeSize::XSmall => 5,
            MazeSize::Small => 10,
            MazeSize::Medium => 20,
            MazeSize::Big => 30,
        }
    }

    pub fn max_spawn_points(&self) -> usize {
        self.room_count() * self.room_count() // one per room
    }
}
