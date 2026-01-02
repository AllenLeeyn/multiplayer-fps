use rand::prelude::SliceRandom;
use rand::{Rng, rng};

use crate::config::{Difficulty, MazeConfig};
use crate::maze::{Cell, Maze};

/// Generates a randomized maze.
/// Each call produces a different maze.
pub fn generate_maze(config: &MazeConfig, name: String) -> Maze {
    let size = config.grid_size();
    let mut maze = Maze::new(name, size, size);
    maze.config = *config;

    let mut rng = rng();

    // 1. Seed a single starting cell (even coordinates)
    let max_x = (size - 1) / 2;
    let max_y = (size - 1) / 2;

    let start_x = rng.random_range(0..=max_x) * 2;
    let start_y = rng.random_range(0..=max_y) * 2;

    maze.set(start_x, start_y, Cell::Empty);

    // 2. DFS corridor carving
    carve_dfs(&mut maze, start_x, start_y, &mut rng);

    // 3. Add loops based on difficulty
    add_loops(&mut maze, config.difficulty, &mut rng);

    // 4. Overlay rooms and extract spawn points
    extract_spawn_points(&mut maze, &mut rng);

    maze
}

fn carve_dfs<R: Rng>(maze: &mut Maze, x: usize, y: usize, rng: &mut R) {
    let mut directions = [(0isize, -2isize), (0, 2), (-2, 0), (2, 0)];
    directions.shuffle(rng);

    for (dx, dy) in directions {
        let nx = x as isize + dx;
        let ny = y as isize + dy;

        if !maze.in_bounds(nx, ny) {
            continue;
        }

        let nx = nx as usize;
        let ny = ny as usize;

        if maze.get(nx, ny) == Cell::Wall {
            let wx = (x + nx) / 2;
            let wy = (y + ny) / 2;

            maze.set(wx, wy, Cell::Empty);
            maze.set(nx, ny, Cell::Empty);

            carve_dfs(maze, nx, ny, rng);
        }
    }
}

fn add_loops<R: Rng>(maze: &mut Maze, difficulty: Difficulty, rng: &mut R) {
    let loop_chance = difficulty.loop_chance();

    for y in 1..maze.height - 1 {
        for x in 1..maze.width - 1 {
            if maze.get(x, y) != Cell::Wall {
                continue;
            }

            if maze.empty_neighbor_count(x, y) >= 2 {
                if rng.random::<f32>() < loop_chance {
                    maze.set(x, y, Cell::Empty);
                }
            }
        }
    }
}

fn extract_spawn_points<R: Rng>(maze: &mut Maze, rng: &mut R) {
    let rooms_per_side = maze.width / 3;

    for ry in 0..rooms_per_side {
        for rx in 0..rooms_per_side {
            let base_x = rx * 3;
            let base_y = ry * 3;

            let mut empty_cells = Vec::new();

            for dy in 0..3 {
                for dx in 0..3 {
                    let x = base_x + dx;
                    let y = base_y + dy;

                    if maze.is_empty(x, y) {
                        empty_cells.push((x, y));
                    }
                }
            }

            if !empty_cells.is_empty() {
                let idx = rng.random_range(0..empty_cells.len());
                let (sx, sy) = empty_cells[idx];
                maze.add_spawn_point(sx, sy);
            } else {
                // Deterministic fallback inside the room
                let cx = base_x + 1;
                let cy = base_y + 1;
                if maze.is_empty(cx, cy) {
                    maze.add_spawn_point(cx, cy);
                }
            }
        }
    }
}
