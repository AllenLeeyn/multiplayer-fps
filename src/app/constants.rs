//! # Constants Module
//!
//! Centralized game configuration constants. Organized by category for easy
//! maintenance and adjustment. All game balance and configuration values should
//! be defined here rather than scattered throughout the codebase.

/// Game tick rate configuration.
///
/// Controls both server tick rate and client input send rate.
/// Change this value to adjust game update frequency (affects responsiveness vs bandwidth).
pub mod tick_rate {
    /// Server and client tick rate in Hz (updates per second).
    /// 
    /// Common values:
    /// - 20 Hz: Lower bandwidth, less responsive (50ms between ticks)
    /// - 30 Hz: Balanced (33ms between ticks) 
    /// - 60 Hz: High responsiveness, more bandwidth (16ms between ticks)
    pub const TICK_RATE_HZ: u32 = 30;
    
    /// Tick duration in milliseconds (derived from tick rate).
    pub const TICK_DURATION_MS: u64 = (1000 / TICK_RATE_HZ) as u64;
    
    /// Tick duration in seconds (for floating-point calculations).
    pub const TICK_DURATION_SECONDS: f32 = 1.0 / (TICK_RATE_HZ as f32);
}

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
    use super::tick_rate;
    
    /// Bullet movement speed in world units per second.
    pub const SPEED: f32 = 1200.0;
    
    /// Cooldown between shots in milliseconds.
    pub const SHOOT_COOLDOWN_MS: u64 = 300;
    
    /// Duration of invincibility after being hit, in seconds.
    pub const INVINCIBILITY_DURATION_SECONDS: f32 = 2.0;
    
    /// Number of substeps for bullet physics calculation based on tick rate.
    /// 
    /// Higher tick rates (smaller dt) need fewer substeps, lower tick rates need more.
    /// Formula: ((60 - tick_rate) + 19) / 20, clamped to [0, 2]
    /// - 60Hz: 0 substeps (dt = 16.67ms, small enough for accurate single-step)
    /// - 30Hz: 2 substeps (dt = 33.33ms, needs subdivision)
    /// - 20Hz: 2 substeps (dt = 50ms, needs subdivision)
    pub const SUBSTEPS: u32 = {
        // Calculate ceiling of (60 - tick_rate) / 20 using integer math
        // Add 19 before division to round up: (n + 19) / 20 = ceil(n / 20)
        let calculated = (60u32.saturating_sub(tick_rate::TICK_RATE_HZ) + 19) / 20;
        // Clamp to [0, 2]
        if calculated > 2 { 2 } else { calculated }
    };
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
    use super::tick_rate;

    /// Default bind address for the game server.
    pub const DEFAULT_BIND_ADDR: &str = "0.0.0.0:9000";
    
    /// Server tick duration in milliseconds (derived from tick rate).
    pub const TICK_DURATION_MS: u64 = tick_rate::TICK_DURATION_MS;
    
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
    use super::tick_rate;

    /// Connection timeout when connecting to a server.
    pub const CONNECTION_TIMEOUT: Duration = Duration::from_secs(5);
    
    /// Sleep duration for the networking thread between polls (milliseconds).
    pub const NET_THREAD_SLEEP_MS: u64 = 16;
    
    /// Interval between ping messages to the server (seconds).
    pub const PING_INTERVAL_SECONDS: u64 = 1;
    
    /// Client input send rate duration (matches server tick rate).
    pub const INPUT_SEND_INTERVAL_MS: u64 = tick_rate::TICK_DURATION_MS;
    
    /// Client input send interval as a `Duration` type.
    pub const INPUT_SEND_INTERVAL: Duration = Duration::from_millis(INPUT_SEND_INTERVAL_MS);
    
    /// Server snapshot interval in seconds (matches server tick rate).
    pub const SNAPSHOT_INTERVAL_SECONDS: f32 = tick_rate::TICK_DURATION_SECONDS;
}
