use serde::{Deserialize, Serialize};
use std::path::Path;
use std::collections::HashMap;

use fps_levels::maze::Maze;

#[derive(Debug, Serialize, Deserialize)]
pub struct Config {
    pub username: String,
    pub maze: Option<Maze>,
    /// Saved server addresses (alias -> address)
    pub saved_servers: HashMap<String, String>,
}

impl Config {
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

    pub fn save<P: AsRef<Path>>(&self, path: P) {
        if let Ok(json) = serde_json::to_string_pretty(self) {
            if let Err(err) = std::fs::write(path, json) {
                eprintln!("Failed to save config: {}", err);
            }
        }
    }
}

impl Default for Config {
    fn default() -> Self {
        Self {
            username: "set username here".to_string(),
            maze: None,
            saved_servers: HashMap::new(),
        }
    }
}
