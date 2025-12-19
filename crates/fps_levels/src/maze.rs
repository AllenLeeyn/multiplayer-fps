use crate::config::{MazeConfig, Difficulty};
use rand::prelude::*;
use rand::rng;

/// Each cell in the maze
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Cell {
    Empty = 0,
    Wall = 1,
}

/// Represents a 2D grid-based maze
#[derive(Debug, Clone)]
pub struct Maze {
    pub config: MazeConfig,
    pub width: usize,
    pub height: usize,
    pub grid: Vec<Vec<Cell>>,
    pub spawn_points: Vec<(usize, usize)>, // (x, y) coordinates
}

impl Maze {
    /// Generate a new maze based on config
    pub fn generate(config: MazeConfig) -> Self {
        let width = config.grid_size();
        let height = width; // square maze

        // Start with a full wall maze
        let mut grid = vec![vec![Cell::Wall; width]; height];

        // Generate the maze using recursive backtracking
        Self::generate_maze(&mut grid, &config);

        // Determine spawn points
        let spawn_points = Self::generate_spawn_points(&grid, &config);

        Maze {
            config,
            width,
            height,
            grid,
            spawn_points,
        }
    }

    /// Maze generation using recursive backtracking
    fn generate_maze(grid: &mut Vec<Vec<Cell>>, config: &MazeConfig) {
        let mut rng = rng();
        let width = grid.len();
        let height = grid[0].len();

        let start_x = rng.random_range(0..width/2) * 2;
        let start_y = rng.random_range(0..height/2) * 2;

        let mut stack = vec![(start_x, start_y)];
        grid[start_y][start_x] = Cell::Empty;

        let directions = [(0, -2), (0, 2), (-2, 0), (2, 0)];

        while let Some((x, y)) = stack.pop() {
            let mut neighbors = Vec::new();

            for &(dx, dy) in &directions {
                let nx = x as isize + dx;
                let ny = y as isize + dy;

                if nx >= 0 && nx < width as isize && ny >= 0 && ny < height as isize {
                    if grid[ny as usize][nx as usize] == Cell::Wall {
                        neighbors.push((nx as usize, ny as usize));
                    }
                }
            }

            if !neighbors.is_empty() {
                stack.push((x, y));

                let &(nx, ny) = neighbors.choose(&mut rng).unwrap();
                let wall_x = (x + nx) / 2;
                let wall_y = (y + ny) / 2;
                grid[wall_y][wall_x] = Cell::Empty;
                grid[ny][nx] = Cell::Empty;

                stack.push((nx, ny));
            }
        }

        Self::add_dead_ends(grid, config.difficulty);
    }

    fn add_dead_ends(grid: &mut Vec<Vec<Cell>>, difficulty: Difficulty) {
        let mut rng = rng();
        let width = grid.len();
        let height = grid[0].len();

        let dead_end_chance = match difficulty {
            Difficulty::Easy => 0.05,
            Difficulty::Normal => 0.15,
            Difficulty::Hard => 0.3,
        };

        for y in 1..height-1 {
            for x in 1..width-1 {
                if grid[y][x] == Cell::Empty && rng.random::<f32>() < dead_end_chance {
                    grid[y][x] = Cell::Wall;
                }
            }
        }
    }

    fn generate_spawn_points(grid: &Vec<Vec<Cell>>, config: &MazeConfig) -> Vec<(usize, usize)> {
        let mut rng = rng();
        let mut spawns = Vec::new();
        let rooms = config.room_count();

        for ry in 0..rooms {
            for rx in 0..rooms {
                let room_x = rx * 3;
                let room_y = ry * 3;

                let mut empty_cells = Vec::new();
                for y in 0..3 {
                    for x in 0..3 {
                        let gx = room_x + x;
                        let gy = room_y + y;
                        if gx < grid.len() && gy < grid[0].len() && grid[gy][gx] == Cell::Empty {
                            empty_cells.push((gx, gy));
                        }
                    }
                }

                if let Some(&spawn) = empty_cells.choose(&mut rng) {
                    spawns.push(spawn);
                }
            }
        }

        spawns
    }

    pub fn is_empty(&self, x: usize, y: usize) -> bool {
        self.grid[y][x] == Cell::Empty
    }
}
