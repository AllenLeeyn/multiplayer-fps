# `fps_levels` – Procedural Maze Generation

A Rust crate for generating procedural mazes for FPS games. It produces **fully connected** mazes with configurable size and difficulty, and supports spawn point assignment via `Maze::set_spawn_points()`.

## Features

- **Procedural generation**: DFS-based corridor carving with configurable loop addition
- **Connectivity guarantee**: All empty cells form a single connected component
- **Spawn point management**: Assign one spawn per 3×3 room with `Maze::set_spawn_points()`
- **Grid geometry**: Cell-based layout suitable for raycasting and collision

## Quick Start

```rust
use fps_levels::{config::MazeConfig, generator::generate_maze};

let config = MazeConfig::new();
let mut maze = generate_maze(&config, "Level 1".to_string());

assert!(maze.is_connected());

// Spawn points are not set by generate_maze(); set them explicitly or call validate()
maze.set_spawn_points().expect("every room has at least one empty cell");

for (x, y) in &maze.spawn_points {
    println!("Spawn at ({}, {})", x, y);
}
```

## Configuration

### Maze Size

```rust
use fps_levels::config::{MazeConfig, MazeSize, Difficulty};

let config = MazeConfig {
    size: MazeSize::Medium,  // 7x7 rooms = 21x21 cells
    difficulty: Difficulty::Normal,
};
```

`MazeConfig::new()` uses size `Small` and difficulty `Custom`.

Available sizes:
- `XSmall`: 3x3 rooms (9x9 cells)
- `Small`: 5x5 rooms (15x15 cells) - **Default**
- `Medium`: 7x7 rooms (21x21 cells)
- `Big`: 9x9 rooms (27x27 cells)

### Difficulty

Difficulty affects loop chance (alternative paths) during procedural generation:
- `Easy`: 25% loop chance
- `Normal`: 15% loop chance - **Default**
- `Hard`: 1% loop chance
- `Custom`: Does not go through `add_loops`; used for user-defined or loaded mazes. The setting has no effect on procedural generation.

## Maze Operations

### Checking Walkability

```rust
// Check if a world position is walkable
let box_size = 100.0;
if maze.is_walkable(player_x, player_y, box_size) {
    // Player can move here
}
```

`Maze::is_walkable()` takes a world position `(x, y)` as `f32` and a `box_size` (cell size in world units). It converts the position to grid coordinates and checks whether the cell at that position is **empty** (walkable); if the cell is a wall, it returns `false`.

### Connectivity Validation

```rust
if !maze.is_connected() {
    panic!("Maze is not fully connected!");
}
```

### Spawn Point Management

```rust
// Automatically set spawn points (one per room)
maze.set_spawn_points()?;

// Or manually add spawn points
maze.add_spawn_point(5, 5);
```

### Validation

`Maze::validate()` runs two checks in order:

1. **Connectivity**: Calls `is_connected()` to ensure all empty cells form a single connected component.
2. **Spawn points**: Calls `set_spawn_points()` to assign one spawn per 3×3 room; returns an error if any room has no empty cells.

Use `validate()` after loading or editing a maze to ensure it is playable before hosting a game.

## Architecture

### Modules

- **`config`**: `MazeConfig`, `MazeSize`, `Difficulty`
- **`generator`**: `generate_maze()` and internal carving/loop logic
- **`maze`**: `Maze`, `Cell`, walkability, connectivity, spawn points, validation

### Types

- **`Cell`**: `Wall` or `Empty`
- **`Maze`**: Grid of cells (row-major), optional spawn list, config; supports Serde

### Generation Algorithm

The `generate_maze()` function implements a multi-step process:

#### Phase 1: Grid Size Selection

**Why odd-number grid sizes are preferred:**

- Odd grid sizes (9, 15, 21, 27) maintain **one-cell wall thickness** while maximizing the playable grid area
- The maze uses a pattern where:
  - **Walls** are placed at odd coordinates (1, 3, 5, ...)
  - **Corridors** are carved at even coordinates (0, 2, 4, ...)
- With an **even grid size**, you would get a full column and row of walls at the boundary, wasting space
- Example: A 15x15 grid (odd) gives you 8×8 = 64 corridor cells, while a 14x14 grid (even) would only give you 7×7 = 49 corridor cells plus boundary walls

#### Phase 2: Starting Cell Selection

- An **empty starting cell** is chosen randomly from all valid even-coordinate positions
- Valid positions are calculated as: `(0..=max_x) * 2` and `(0..=max_y) * 2` where `max_x = (size - 1) / 2`
- This ensures the starting point is always at an even coordinate, aligned with the corridor pattern

#### Phase 3: DFS Corridor Carving (`carve_dfs`)

The depth-first search algorithm carves paths through the maze:

1. **Random direction selection**: At each cell, the four possible directions (up, down, left, right) are shuffled randomly
2. **Two-cell jumps**: The algorithm moves **two cells** at a time (skipping over walls) to maintain the even-coordinate pattern
3. **Wall removal**: When moving from `(x, y)` to `(nx, ny)`, **walls are only carved if `(nx, ny)` is a wall** (unexplored area):
   - Carves the wall between them at `((x + nx) / 2, (y + ny) / 2)`
   - Carves the destination cell at `(nx, ny)`
   - This condition prevents re-carving already explored paths and ensures DFS only expands into new (wall) territory
   - **Dead end creation**: When a cell has no unexplored neighbors (all `(nx, ny)` are already empty), no further carving occurs at that cell, naturally creating a dead end
4. **Recursive exploration**: After carving, it recursively explores from the new position
5. **Backtracking**: When a dead-end is reached (all neighbors are already explored/empty), the recursion unwinds, returning to previous cells to explore other possible directions
6. **Guaranteed connectivity**: Because DFS visits every reachable cell, all carved corridors form a single connected component

#### Phase 4: Loop Addition (`add_loops`)

After the initial DFS carving creates a perfect tree (no loops), `add_loops()` adds alternative paths:

1. **Wall scanning**: Iterates through all wall cells in the interior of the maze (excluding boundaries)
2. **Neighbor check**: For each wall, counts how many of its 4-connected neighbors are empty cells
3. **Loop creation**: If a wall has **2 or more empty neighbors**, it's a candidate for removal (creating a loop)
4. **Probability-based removal**: Each candidate wall is removed with a probability based on difficulty (Easy 25%, Normal 15%, Hard 1%). Custom difficulty does not affect this step; see [Difficulty](#difficulty).
5. **Result**: Alternative paths are created; the maze stays fully connected.

#### Phase 5: Spawn Points

- `generate_maze()` does **not** set spawn points; the returned maze has an empty `spawn_points` list.
- Call `Maze::set_spawn_points()` (or `Maze::validate()`) to assign one spawn per 3×3 room; each room’s spawn is chosen randomly from its empty cells.

## Coordinate System

- Origin `(0, 0)` is at the top-left corner
- `x` increases to the right
- `y` increases downward
- Cells are stored in row-major order: `cells[y * width + x]`

## Serialization

`Maze` and `Cell` implement `Serialize` and `Deserialize` from Serde. Use with any format (e.g. `serde_json` in your crate):

```rust
let json = serde_json::to_string(&maze)?;
let loaded: Maze = serde_json::from_str(&json)?;
```

## License

Part of the multiplayer-fps project.
