//! # fps_config
//!
//! Small helper crate that owns the **persistent configuration** for the
//! multiplayer FPS game.
//!
//! The configuration is:
//! - Stored as a JSON file on disk
//! - Loaded on startup (falling back to sensible defaults)
//! - Saved whenever the user changes relevant settings (e.g. username, maze,
//!   saved servers)
//!
//! The main binary uses [`Config::load`] and [`Config::save`] to keep things
//! like the current username, last selected maze and a list of favourite
//! servers consistent between runs.
 
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::Path;
 
use fps_levels::maze::Maze;
 
/// Top‑level configuration for the game.
///
/// This struct is serialized to / deserialized from JSON and is intended to be
/// cheap to clone and pass around the UI / game layers.
#[derive(Debug, Serialize, Deserialize)]
pub struct Config {
    /// The name shown in the UI and used as the player's identifier when
    /// connecting to a server.
    pub username: String,

    /// The last maze configuration that was loaded / edited.
    ///
    /// When `None`, the game will fall back to a default maze selection.
    pub maze: Option<Maze>,

    /// Saved server addresses, keyed by a human‑friendly alias.
    ///
    /// Example entries:
    ///
    /// - `"Home" -> "192.168.1.10:9000"`
    /// - `"Mobile Hotspot" -> "10.0.0.5:9000"`
    pub saved_servers: HashMap<String, String>,
}
 
impl Config {
    /// Load configuration from a JSON file.
    ///
    /// If the file is missing or cannot be parsed, this logs a message and
    /// returns [`Config::default`] instead of crashing, so the game can still
    /// start.
    pub fn load<P: AsRef<Path>>(path: P) -> Self {
        match std::fs::read_to_string(&path) {
            Ok(contents) => serde_json::from_str(&contents).unwrap_or_else(|err| {
                eprintln!("Failed to parse config: {}. Using default.", err);
                Self::default()
            }),
            Err(_) => {
                println!(
                    "Config file not found at {:?}. Using default.",
                    path.as_ref()
                );
                Self::default()
            }
        }
    }
 
    /// Save the current configuration back to disk as pretty‑printed JSON.
    ///
    /// Any I/O or serialization errors are logged to stderr and otherwise
    /// ignored – failure to persist config should not crash the game.
    pub fn save<P: AsRef<Path>>(&self, path: P) {
        if let Ok(json) = serde_json::to_string_pretty(self) {
            if let Err(err) = std::fs::write(path, json) {
                eprintln!("Failed to save config: {}", err);
            }
        }
    }
}
 
impl Default for Config {
    /// Construct a minimal default configuration used when no config file
    /// exists yet or the existing one cannot be parsed.
    fn default() -> Self {
        Self {
            username: "set username here".to_string(),
            maze: None,
            saved_servers: HashMap::new(),
        }
    }
}
