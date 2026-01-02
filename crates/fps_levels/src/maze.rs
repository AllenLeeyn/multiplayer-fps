use rand::Rng;
use serde::{Deserialize, Serialize};
use std::collections::VecDeque;
use std::error::Error;

use super::config::MazeConfig;

/// A single maze cell.
/// Wall blocks movement and rays.
/// Empty is fully walkable.
#[derive(Debug, Copy, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[repr(u8)]
pub enum Cell {
    Wall = 0,
    Empty = 1,
}

/// A generated maze level.
///
/// The maze is a rectangular grid stored in row-major order.
/// Coordinate system:
/// - (0, 0) is the top-left corner
/// - x increases to the right
/// - y increases downward
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Maze {
    pub name: String,
    pub width: usize,
    pub height: usize,
    pub cells: Vec<Cell>, // row-major: y * width + x
    pub spawn_points: Vec<(usize, usize)>,
    pub config: MazeConfig,
}

impl Maze {
    /// Creates a new maze filled entirely with walls.
    ///
    /// Invariant:
    /// - cells.len() == width * height
    pub fn new(name: String, width: usize, height: usize) -> Self {
        let cells = vec![Cell::Wall; width * height];

        Self {
            name,
            width,
            height,
            cells,
            spawn_points: Vec::new(),
            config: MazeConfig::new(),
        }
    }

    /// Returns true if the given coordinates are inside the maze bounds.
    #[inline]
    pub fn in_bounds(&self, x: isize, y: isize) -> bool {
        x >= 0 && y >= 0 && (x as usize) < self.width && (y as usize) < self.height
    }

    /// Converts (x, y) into a linear index.
    ///
    /// # Panics
    /// Panics if (x, y) is out of bounds.
    #[inline]
    pub fn index(&self, x: usize, y: usize) -> usize {
        debug_assert!(x < self.width && y < self.height);
        y * self.width + x
    }

    /// Returns the cell at (x, y).
    ///
    /// # Panics
    /// Panics if (x, y) is out of bounds.
    #[inline]
    pub fn get(&self, x: usize, y: usize) -> Cell {
        let idx = self.index(x, y);
        self.cells[idx]
    }

    /// Sets the cell at (x, y).
    ///
    /// # Panics
    /// Panics if (x, y) is out of bounds.
    #[inline]
    pub fn set(&mut self, x: usize, y: usize, cell: Cell) {
        let idx = self.index(x, y);
        self.cells[idx] = cell;
    }

    /// Returns true if the cell at (x, y) is empty.
    #[inline]
    pub fn is_empty(&self, x: usize, y: usize) -> bool {
        self.get(x, y) == Cell::Empty
    }

    /// Returns all in-bounds 4-connected neighbors of (x, y).
    pub fn neighbors_4(&self, x: usize, y: usize) -> impl Iterator<Item = (usize, usize)> {
        const DIRS: [(isize, isize); 4] = [
            (0, -1), // up
            (0, 1),  // down
            (-1, 0), // left
            (1, 0),  // right
        ];

        DIRS.into_iter().filter_map(move |(dx, dy)| {
            let nx = x as isize + dx;
            let ny = y as isize + dy;
            if self.in_bounds(nx, ny) {
                Some((nx as usize, ny as usize))
            } else {
                None
            }
        })
    }

    /// Counts how many 4-connected neighbors of (x, y) are empty.
    pub fn empty_neighbor_count(&self, x: usize, y: usize) -> usize {
        self.neighbors_4(x, y)
            .filter(|&(nx, ny)| self.is_empty(nx, ny))
            .count()
    }

    /// Adds a spawn point.
    ///
    /// # Panics
    /// Panics if the spawn point is out of bounds or not on an empty cell.
    pub fn add_spawn_point(&mut self, x: usize, y: usize) {
        debug_assert!(x < self.width && y < self.height);
        debug_assert!(self.is_empty(x, y));

        self.spawn_points.push((x, y));
    }

    /// Returns true if all empty cells form a single connected component.
    ///
    /// Temporary `u8` clone is used:
    /// - 0 = Wall
    /// - 1 = Empty
    /// - 2 = Visited Empty
    pub fn is_connected(&self) -> bool {
        // Clone maze into u8 grid
        let mut grid: Vec<u8> = self.cells.iter().map(|&c| c as u8).collect();

        // Find the first empty cell to start BFS
        let start_idx = match grid.iter().position(|&v| v == 1) {
            Some(idx) => idx,
            None => return false, // no empty cells → invalid
        };

        let start_x = start_idx % self.width;
        let start_y = start_idx / self.width;

        let mut queue = VecDeque::new();
        queue.push_back((start_x, start_y));
        grid[start_idx] = 2; // mark as visited

        // BFS flood-fill
        while let Some((x, y)) = queue.pop_front() {
            for (nx, ny) in self.neighbors_4(x, y) {
                let idx = self.index(nx, ny);
                if grid[idx] == 1 {
                    grid[idx] = 2; // mark visited
                    queue.push_back((nx, ny));
                }
            }
        }

        // If any empty cell remains unvisited, the maze is disconnected
        !grid.iter().any(|&v| v == 1)
    }

    pub fn set_spawn_points(&mut self) -> Result<(), Box<dyn Error>> {
        let room_size = 3;
        let rooms_x = self.width / room_size;
        let rooms_y = self.height / room_size;

        let mut rng = rand::rng();
        self.spawn_points.clear(); // reset existing points

        for ry in 0..rooms_y {
            for rx in 0..rooms_x {
                let mut empty_cells = Vec::new();

                // Collect all empty cells in this room
                for y in (ry * room_size)..((ry + 1) * room_size) {
                    for x in (rx * room_size)..((rx + 1) * room_size) {
                        if self.is_empty(x, y) {
                            empty_cells.push((x, y));
                        }
                    }
                }

                if empty_cells.is_empty() {
                    return Err(format!(
                        "Room at ({}, {}) has no empty cells for a spawn point",
                        rx, ry
                    )
                    .into());
                }

                // Randomly pick one empty cell as spawn
                let idx = rng.random_range(0..empty_cells.len());
                let spawn = empty_cells[idx];
                self.spawn_points.push(spawn);
            }
        }

        Ok(())
    }

    pub fn validate(&mut self) -> Result<(), String> {
        // 1. Check connectivity
        if !self.is_connected() {
            return Err("Maze is not fully connected".to_string());
        }

        // 2. Try to set spawn points
        self.set_spawn_points()
            .map_err(|e| format!("Failed to set spawn points: {}", e))?;

        Ok(())
    }
}

use std::fmt;

impl fmt::Display for Maze {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for y in 0..self.height {
            for x in 0..self.width {
                let ch = match self.get(x, y) {
                    Cell::Wall => '#',
                    Cell::Empty => {
                        if self.spawn_points.contains(&(x, y)) {
                            'S'
                        } else {
                            '.'
                        }
                    }
                };
                write!(f, "{ch}")?;
            }
            writeln!(f)?;
        }
        Ok(())
    }
}
