/// Represents the number of rooms per side in the maze.
#[derive(Debug, Clone, Copy)]
pub enum MazeSize {
    Small,  // 4x4 rooms
    Medium, // 5x5 rooms
    Big,    // 6x6 rooms
}

impl MazeSize {
    /// Returns the number of rooms per side
    pub fn room_count(&self) -> usize {
        match self {
            MazeSize::Small => 4,
            MazeSize::Medium => 5,
            MazeSize::Big => 6,
        }
    }

    /// Returns the total grid size in cells (each room is 3x3)
    pub fn grid_size(&self) -> usize {
        self.room_count() * 3
    }
}

#[derive(Debug, Clone, Copy)]
pub enum Difficulty {
    Easy,
    Normal,
    Hard,
}

#[derive(Debug, Clone, Copy)]
pub struct MazeConfig {
    pub size: MazeSize,
    pub difficulty: Difficulty,
}

impl MazeConfig {
    pub fn room_count(&self) -> usize {
        self.size.room_count()
    }

    pub fn grid_size(&self) -> usize {
        self.size.grid_size()
    }

    pub fn max_players(&self) -> usize {
        match self.size {
            MazeSize::Small => 10,
            MazeSize::Medium => 20,
            MazeSize::Big => 30,
        }
    }

    pub fn max_spawn_points(&self) -> usize {
        self.room_count() * self.room_count() // one per room
    }
}
