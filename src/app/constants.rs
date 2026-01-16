//! # Constants Module
//!
//! Centralized game configuration constants. Organized by category for easy
//! maintenance and adjustment. All game balance and configuration values should
//! be defined here rather than scattered throughout the codebase.

/// Game physics and world constants.
///
/// Defines sizes and dimensions for game entities in world units.
pub mod physics {
    /// Size of each maze cell (wall box) in world units.
    pub const BOX_SIZE: f32 = 100.0;
    
    /// Size of player entities in world units.
    pub const PLAYER_SIZE: f32 = 30.0;
    
    /// Size of bullet projectiles in world units.
    pub const BULLET_SIZE: f32 = 5.0;
}

/// Player movement constants.
///
/// Defines speeds, movement modifiers, and input sensitivity for players.
pub mod player {
    /// Base walking speed in world units per second.
    pub const WALK_SPEED: f32 = 250.0;
    
    /// Running speed (with Shift held) in world units per second.
    pub const RUN_SPEED: f32 = 350.0;
    
    /// Speed multiplier for sidestepping (A/D keys).
    pub const SIDESTEP_FACTOR: f32 = 0.8;
    
    /// Speed multiplier for moving backward (S key).
    pub const BACKSTEP_FACTOR: f32 = 0.7;
    
    /// Speed multiplier applied when player is invincible.
    pub const INVINCIBLE_SPEED_MULTIPLIER: f32 = 1.5;
    
    /// Mouse sensitivity for rotation (applied to mouse delta).
    pub const ROTATION_SENSITIVITY: f32 = 0.05;
}

/// Bullet and shooting constants.
///
/// Defines bullet physics and shooting mechanics.
pub mod bullet {
    /// Bullet movement speed in world units per second.
    pub const SPEED: f32 = 1200.0;
    
    /// Cooldown between shots in milliseconds.
    pub const SHOOT_COOLDOWN_MS: u64 = 300;
    
    /// Duration of invincibility after being hit, in seconds.
    pub const INVINCIBILITY_DURATION_SECONDS: f32 = 2.0;
}

/// Game scoring constants.
///
/// Defines point values and score requirements.
pub mod scoring {
    /// Points deducted when a player is hit.
    pub const HIT_PENALTY: u32 = 5;
    
    /// Points awarded when a player hits an opponent.
    pub const HIT_REWARD: u32 = 10;
    
    /// Minimum target score for winning a game.
    pub const MIN_TARGET_SCORE: u32 = 30;
    
    /// Maximum target score for winning a game.
    pub const MAX_TARGET_SCORE: u32 = 9999;
}

/// Server configuration constants.
///
/// Defines server networking and tick rate settings.
pub mod server {
    use std::time::Duration;

    /// Default bind address for the game server.
    pub const DEFAULT_BIND_ADDR: &str = "0.0.0.0:9000";
    
    /// Server tick duration in milliseconds (~60 FPS).
    pub const TICK_DURATION_MS: u64 = 16;
    
    /// Server tick duration as a `Duration` type.
    pub const TICK_DURATION: Duration = Duration::from_millis(TICK_DURATION_MS);
    
    /// Client timeout duration in seconds before disconnection.
    pub const CLIENT_TIMEOUT_SECONDS: u64 = 5;
}

/// Client configuration constants.
///
/// Defines client networking and connection settings.
pub mod client {
    use std::time::Duration;

    /// Connection timeout when connecting to a server.
    pub const CONNECTION_TIMEOUT: Duration = Duration::from_secs(5);
    
    /// Sleep duration for the networking thread between polls (milliseconds).
    pub const NET_THREAD_SLEEP_MS: u64 = 16;
    
    /// Interval between ping messages to the server (seconds).
    pub const PING_INTERVAL_SECONDS: u64 = 1;
}
