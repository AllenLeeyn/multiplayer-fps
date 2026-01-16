//! # Application Module
//!
//! Core application logic for the multiplayer FPS game. This module contains:
//!
//! - **App**: Main application struct managing UI, views, and game state
//! - **Game Logic**: Client/server game state management
//! - **Networking**: Thread-safe communication between game and network layers
//! - **Client Management**: Player state and connection tracking
//! - **Game Entities**: Players, bullets, positions, and game states
//! - **Input Handling**: Player input state and processing
//! - **Constants**: Centralized game configuration values
//! - **View IDs**: Centralized UI component and view identifiers
//!
//! ## Architecture Overview
//!
//! The application follows a layered architecture:
//!
//! ```
//! UI Layer (fps_ui)
//!   ↓ UIEvent
//! View Layer (src/view)
//!   ↓ ViewAction
//! App Layer (src/app/app.rs)
//!   ↓ GameNetCommand / GameNetEvent
//! Game Logic (src/app/game_client.rs, game_server.rs)
//!   ↓ UDP Messages
//! Network Layer (fps_net)
//! ```
//!
//! ## Module Structure
//!
//! - **`app.rs`**: Main application entry point and event loop
//! - **`client.rs`**: Client connection state and management
//! - **`game.rs`**: Core game logic (player updates, bullet physics, collisions)
//! - **`game_client.rs`**: Client-side game state and networking integration
//! - **`game_client_net.rs`**: Networking thread for client communication
//! - **`game_server.rs`**: Server-side game loop and client management
//! - **`game_structs.rs`**: Game state types and enums
//! - **`game_input.rs`**: Player input state tracking
//! - **`pos.rs`**: Position and rotation utilities
//! - **`constants.rs`**: Game configuration constants
//! - **`view_ids.rs`**: UI view and component identifier constants

pub mod app;
pub mod client;
pub mod constants;
pub mod game;
pub mod game_client;
pub mod game_client_net;
pub mod game_server;
pub mod game_structs;
pub mod game_input;
pub mod pos;
pub mod view_ids;

pub use app::App;
pub use client::{Client, ClientList, ClientStatus};
pub use game_server::{GameServer, ServerHandle};
pub use game_structs::{GameState, PlayerAction};
pub use game_input::GameInputState;
pub use pos::Pos;
