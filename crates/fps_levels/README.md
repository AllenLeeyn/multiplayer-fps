# `fps_levels` Crate — Specification Sheet

## Purpose

`fps_levels` is responsible for **procedural maze generation and level data management** for the multiplayer FPS.
It guarantees **fully connected, deterministic mazes** and provides **structured spawn point extraction** without affecting maze topology.

The crate contains **no rendering, networking, or gameplay logic**.

---

## Design Principles

1. **Connectivity by Construction**

   * The maze generator must never produce isolated walkable areas.
   * Connectivity is guaranteed by generation rules, not by validation passes.

2. **Deterministic Output**

   * Given the same configuration and seed, the generated level is byte-identical.
   * Required for server-authoritative networking.

3. **Separation of Concerns**

   * Maze topology is generated first.
   * Rooms exist only as a post-processing abstraction for spawn points.

4. **FPS-Oriented Geometry**

   * Grid-based corridors suitable for raycasting.
   * Uniform wall height, flat floor.

---

## Public API

### Modules

```
fps_levels/
├── maze.rs       // Level & Cell definitions
├── generator.rs  // Maze generation logic
├── config.rs     // MazeConfig, enums
├── lib.rs
```

---

## Core Data Structures

### `Cell`

```rust
#[derive(Copy, Clone, Serialize, Deserialize, PartialEq)]
pub enum Cell {
    Wall,
    Empty,
}
```

---

### `Maze`

```rust
#[derive(Serialize, Deserialize)]
pub struct Maze {
    pub name: String,
    pub width: u32,              // cell width
    pub height: u32,             // cell height
    pub cells: Vec<Cell>,        // row-major grid
    pub spawn_points: Vec<(u32, u32)>,
}
```

#### Guarantees

* All `Empty` cells form a **single connected component**
* All spawn points are reachable from one another

---

## Configuration Types

### `MazeSize`

```rust
pub enum MazeSize {
    Small,   // 4x4 rooms → 12x12 cells → 10 players max
    Medium,  // 5x5 rooms → 15x15 cells → 20 players max
    Big,     // 6x6 rooms → 18x18 cells → 30 players max
}
```

---

### `Difficulty`

```rust
pub enum Difficulty {
    Easy,    // few dead ends
    Normal,  // balanced
    Hard,    // many dead ends
}
```

---

### `MazeConfig`

```rust
pub struct MazeConfig {
    pub size: MazeSize,
    pub difficulty: Difficulty,
    pub seed: u64,
}
```

---

## Maze Geometry Rules

* Maze is a **cell grid**
* Each cell is **1×1 world units**
* Players move freely in `Empty` cells
* Walls fully block movement and rays

---

## Maze Generation Algorithm

### Generation Pipeline

1. Initialize grid (all walls)
2. Seed a single empty cell
3. Carve corridors using **Depth-First Search**
4. Add loops using **Rule 1–safe carving**
5. Overlay room grid to extract spawn points

---

### Connectivity Rule (Rule 1)

> A wall cell may only be carved if it is adjacent to at least one empty cell.

This rule is **never violated**.

---

### DFS Corridor Carving

* Carving advances in steps of 2 cells
* Walls remain between corridors
* Ensures a perfect maze (tree) initially

---

### Loop Creation (Difficulty Control)

Loops are created by converting walls to empty **only if**:

* The wall is adjacent to **two or more empty neighbors**
* Random chance passes (difficulty dependent)

| Difficulty | Loop Chance | Result         |
| ---------- | ----------- | -------------- |
| Easy       | High        | Few dead ends  |
| Normal     | Medium      | Balanced       |
| Hard       | Low         | Many dead ends |

---

## Room Overlay & Spawn Points

### Room Definition

* A room is a **3×3 cell block**
* Rooms are derived **after** maze generation
* Rooms do **not** affect maze topology

---

### Room Grid by Maze Size

| Maze Size | Rooms | Cells | Spawn Points |
| --------- | ----- | ----- | ------------ |
| Small     | 4×4   | 12×12 | 16           |
| Medium    | 5×5   | 15×15 | 25           |
| Big       | 6×6   | 18×18 | 36           |

---

### Spawn Point Rules

* Each room is a 3×3 grid of cells.
* For each room, **randomly select one `Empty` cell** to serve as the spawn point.
* If the room has no `Empty` cells (extremely rare if maze generation is correct), skip or fallback to the center cell.
* Spawn points are **evenly distributed across all rooms**, so each room contributes **exactly one spawn point**.
* All spawn points remain **mutually reachable**, guaranteed by maze connectivity.
* The server can **shuffle spawn points** at the start of the match for added variability.

---

## Guarantees Provided by `fps_levels`

✔ No isolated walkable areas
✔ No unreachable spawn points
✔ Deterministic generation
✔ Difficulty affects topology, not randomness
✔ Suitable for raycasting FPS rendering
✔ Server-authoritative safe

---

## Non-Responsibilities

The crate explicitly does **not** handle:

* Rendering
* Physics
* Collision detection
* Networking
* Player logic
* AI behavior

---

## Expected Consumers

* **Server**

  * Generates level once
  * Sends serialized `Level` to clients
* **Client**

  * Receives level
  * Uses it for rendering, collision, minimap

---

## Future Extensions (Non-Breaking)

* Spawn point scoring (avoid dead ends)
* Themed wall variations
* Multi-floor levels (z-layer extension)
* Editor-driven maze overrides

---

## Invariants (Must Always Hold)

1. `cells.len() == width * height`
2. At least one `Cell::Empty`
3. All `Cell::Empty` cells are connected
4. All spawn points lie on `Cell::Empty`
5. Level generation never fails

---

## Summary

The `fps_levels` crate provides a **robust, deterministic, FPS-optimized maze generation system** that guarantees connectivity and clean spawn distribution while remaining simple, testable, and network-safe.
