pub mod app;
pub mod client;
pub mod game;
pub mod game_client;
pub mod game_client_net;
pub mod game_server;
pub mod game_structs;
pub mod game_input;
pub mod pos;

pub use app::App;
pub use client::{Client, ClientList, ClientStatus};
pub use game_server::{GameServer, ServerHandle};
pub use game_structs::{GameState, PlayerAction};
pub use game_input::GameInputState;
pub use pos::Pos;

pub const BOX_SIZE: f32 = 100.0;
pub const PLAYER_SIZE: f32 = 50.0;
pub const PLAYER_WALK_SPD: f32 = 250.0;
pub const PLAYER_RUN_SPD: f32 = 350.0;
pub const PLAYER_SIDESTEP_FACTOR: f32 = 0.8;
pub const PLAYER_BACKSTEP_FACTOR: f32 = 0.7;
pub const BULLET_SIZE: f32 = 5.0;
pub const BULLET_SPD: f32 = 1200.0;
