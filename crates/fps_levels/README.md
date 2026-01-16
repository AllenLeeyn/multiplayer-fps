# `fps_levels` - Procedural Maze Generation

A Rust crate for generating procedural mazes optimized for FPS games. Guarantees fully connected mazes with configurable difficulty and spawn point extraction.

## Features

- **Procedural Generation**: DFS-based maze generation with configurable loop addition
- **Connectivity Guarantee**: All empty cells form a single connected component
- **Deterministic**: Same configuration produces identical mazes (important for networking)
- **Spawn Point Management**: Automatic extraction of spawn points from room regions
- **FPS-Optimized**: Grid-based geometry suitable for raycasting

## Quick Start

```rust
use fps_levels::{config::MazeConfig, generator::generate_maze};

// Create a default configuration
let config = MazeConfig::new();

// Generate a maze
let maze = generate_maze(&config, "Level 1".to_string());

// Check connectivity
assert!(maze.is_connected());

// Access spawn points
for (x, y) in &maze.spawn_points {
    println!("Spawn point at ({}, {})", x, y);
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

Available sizes:
- `XSmall`: 3x3 rooms (9x9 cells)
- `Small`: 5x5 rooms (15x15 cells) - **Default**
- `Medium`: 7x7 rooms (21x21 cells)
- `Big`: 9x9 rooms (27x27 cells)

### Difficulty

Difficulty affects loop chance (alternative paths):
- `Easy`: 25% loop chance
- `Normal`: 15% loop chance - **Default**
- `Hard`: 1% loop chance
- `Custom`: 10% loop chance

## Maze Operations

### Checking Walkability

```rust
// Check if a world position is walkable
let box_size = 100.0;
if maze.is_walkable(player_x, player_y, box_size) {
    // Player can move here
}
```

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

## Architecture

### Modules

- **`config`**: Configuration types (`MazeConfig`, `MazeSize`, `Difficulty`)
- **`generator`**: Procedural generation algorithms
- **`maze`**: Core data structures (`Maze`, `Cell`)

### Generation Algorithm

1. **Seed**: Start with a random even-coordinate cell
2. **DFS Carving**: Carve corridors using depth-first search (guarantees connectivity)
3. **Loop Addition**: Add loops based on difficulty settings
4. **Spawn Extraction**: Extract spawn points from 3x3 room regions

## Coordinate System

- Origin `(0, 0)` is at the top-left corner
- `x` increases to the right
- `y` increases downward
- Cells are stored in row-major order: `cells[y * width + x]`

## Serialization

Mazes can be serialized/deserialized using Serde:

```rust
use serde_json;

let json = serde_json::to_string(&maze)?;
let loaded_maze: Maze = serde_json::from_str(&json)?;
```

## License

Part of the multiplayer-fps project.
