//! # `fps_levels` - Procedural Maze Generation and Level Management
//!
//! This crate provides procedural maze generation and level data management for multiplayer FPS games.
//! It guarantees fully connected, deterministic mazes and provides structured spawn point extraction.
//!
//! ## Design Principles
//!
//! 1. **Connectivity by Construction**: The maze generator never produces isolated walkable areas.
//!    Connectivity is guaranteed by generation rules, not by validation passes.
//!
//! 2. **Deterministic Output**: Given the same configuration and seed, the generated level is byte-identical.
//!    Required for server-authoritative networking.
//!
//! 3. **Separation of Concerns**: Maze topology is generated first. Rooms exist only as a post-processing
//!    abstraction for spawn points.
//!
//! 4. **FPS-Oriented Geometry**: Grid-based corridors suitable for raycasting with uniform wall height
//!    and flat floor.
//!
//! ## Modules
//!
//! - [`config`]: Configuration types for maze size and difficulty
//! - [`generator`]: Procedural maze generation algorithms
//! - [`maze`]: Core maze data structures and operations
//!
//! ## Example
//!
//! ```rust,no_run
//! use fps_levels::{config::MazeConfig, generator::generate_maze};
//!
//! let config = MazeConfig::new();
//! let maze = generate_maze(&config, "My Maze".to_string());
//! ```

pub mod config;
pub mod generator;
pub mod maze;

pub use config::{Difficulty, MazeConfig, MazeSize};
pub use generator::generate_maze;
pub use maze::{Cell, Maze};
