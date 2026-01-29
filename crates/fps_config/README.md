# fps_config

Small helper crate that owns the **persistent configuration** for the
multiplayer FPS game.

It centralises everything the game needs to remember between runs:

- Current username
- Last selected / edited maze
- Saved server addresses under human‑friendly aliases

The main binary calls into this crate to **load** the configuration at startup
and **save** it again whenever settings change.

---

## Data model

The crate exposes a single public type:

- `Config`
  - `username: String`
    - Name shown in the UI and used as the player's identifier when connecting
      to a server.
  - `maze: Option<fps_levels::maze::Maze>`
    - Optional snapshot of the maze configuration that should be used on the
      next host session (or last one that was edited).
  - `saved_servers: HashMap<String, String>`
    - Mapping from alias → `"ip:port"` string.
    - Example:
      - `"Home" -> "192.168.1.137:9000"`
      - `"Mobile" -> "10.61.159.225:9000"`

The JSON file consumed by the main game is compatible with this structure.
An example (simplified) `config.json` looks like:

```json
{
  "username": "Leee",
  "maze": {
    "name": "OPEN WARS",
    "width": 9,
    "height": 9,
    "cells": [ /* maze cell data */ ],
    "spawn_points": [],
    "config": {
      "size": "Small",
      "difficulty": "Custom"
    }
  },
  "saved_servers": {
    "Mobile": "10.61.159.225:9000",
    "Home": "192.168.1.137:9000"
  }
}
```

---

## Serialization with Serde

The crate uses **[serde](https://serde.rs/)** and **[serde_json](https://docs.serde.rs/serde_json/)** to handle JSON serialization and deserialization.

### Dependencies

- **`serde`** (with `derive` feature): Provides the `Serialize` and `Deserialize` traits
- **`serde_json`**: JSON-specific serialization format for serde

### Why `derive` is required

The `derive` feature enables **procedural macros** that automatically generate implementations of `Serialize` and `Deserialize` traits. Without it, you would need to manually implement these traits, which is verbose and error-prone.

**With `derive` feature** (what we use):
```rust
#[derive(Serialize, Deserialize)]  // ← This macro generates the trait impls
pub struct Config { ... }
```

**Without `derive` feature** (manual implementation):
```rust
// You'd have to write hundreds of lines of boilerplate like this:
impl Serialize for Config {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where S: Serializer {
        let mut state = serializer.serialize_struct("Config", 3)?;
        state.serialize_field("username", &self.username)?;
        state.serialize_field("maze", &self.maze)?;
        state.serialize_field("saved_servers", &self.saved_servers)?;
        state.end()
    }
}
// ... and similar for Deserialize
```

The `derive` feature is essential because:
- It generates all the serialization/deserialization code automatically
- It handles complex nested types (`Option`, `HashMap`, custom types like `Maze`) without manual work
- It keeps the codebase maintainable: adding/removing fields only requires updating the struct definition

### How it works

The `Config` struct derives both `Serialize` and `Deserialize`:

```rust
#[derive(Debug, Serialize, Deserialize)]
pub struct Config {
    pub username: String,
    pub maze: Option<Maze>,
    pub saved_servers: HashMap<String, String>,
}
```

- **`Serialize`**: Enables converting `Config` → JSON string
  - Used by `Config::save()` via `serde_json::to_string_pretty()`
  - Automatically handles nested types like `Option<Maze>` and `HashMap`
  
- **`Deserialize`**: Enables converting JSON string → `Config`
  - Used by `Config::load()` via `serde_json::from_str()`
  - Automatically parses JSON and reconstructs the Rust struct

The derive macros handle all the boilerplate code needed to convert between Rust types and JSON. This means:
- No manual JSON parsing/writing code needed
- Type-safe: invalid JSON structure will fail at compile time (for the struct definition) or runtime (during parsing)
- Works seamlessly with nested types (`Maze`, `HashMap`, `Option`, etc.)

---

## API overview

### `Config::load`

```rust
use fps_config::Config;

let config_path = "src/assets/config.json";
let config = Config::load(config_path);
```

- Attempts to read and parse JSON from the given path.
- On missing file or parse error:
  - Logs a message.
  - Returns `Config::default()` so the game can still start.

### `Config::save`

```rust
config.save(config_path);
```

- Serialises the config as **pretty‑printed JSON** and writes it to disk.
- Logs (but ignores) any I/O errors; saving should not crash the game loop.

### `Config::default`

Used implicitly by `Config::load` when there is no usable config file yet.

Defaults:

- `username`: `"set username here"`
- `maze`: `None`
- `saved_servers`: empty map

---

## How it is used in the game

In `src/main.rs`:

- On startup:
  - The game calls `Config::load("src/assets/config.json")`.
  - The resulting `Config` is stored in the top‑level `App` struct.
- While running:
  - Views (main menu, host menu, lobby, etc.) read from and update the `Config`
    (e.g. username, chosen maze, server list).
- On changes:
  - The app calls `config.save(config_path)` so that the next run starts with
    the same preferences and server history.

