# multiplayer_fps

## Project Description
### **Project Overview: Real-Time Networked Multiplayer FPS (Maze Wars Inspired)**

This project develops a real-time, networked multiplayer first-person shooter (FPS) game inspired by "Maze Wars." The core innovation is a single application executable that can operate in two modes: either host a game (starting a server and automatically joining it as a client) or join an existing game (connecting as a client). The game features intricate, flat, grid-based mazes, projectile combat, a scoring system, and distinct player roles ("Spheres" and "Cubes" with unique abilities). It will utilize UDP for low-latency communication, support 10 to 30 concurrent players, and enable cross-platform connectivity.

---

### **Requirements:**

**Application Modes:** Single executable must support "Host Game" (starts server, auto-joins as client) and "Join Game" (client only).

**Core Gameplay Mechanics:**
*   **Maze Navigation:** Players navigate a flat, grid-based maze environment composed of empty spaces and solid walls of uniform height.
*   **Projectile Combat:** Players engage in projectile-based shooting.
*   **Scoring System:**
    *   10 points awarded for a successful hit on an opponent.
    *   -5 points deducted for being hit by an opponent.
*   **Combat Grace Period:** A 2-second grace period before a player can be shot again after being hit.
*   **Winning Condition:** The game ends when a player reaches a pre-defined winning score.
*   **Leaderboard:** A dynamic leaderboard displays current player standings.
**Networking and Server Infrastructure:**

*   **Client-Server Model:** Establish a robust client-server architecture.
*   **Communication Protocol:** Employ UDP for efficient, low-latency communication.
*   **Server Capacity:** The server must handle between 10 and 30 concurrent client connections.
*   **Cross-Platform Play:** Enable clients to connect to a server running on a separate machine.

**User Interface (UI) and User Experience (UX):**

*   **Mini-Map:** Display an accurate representation of the player's current position and the overall game world layout.
*   **Visual Graphics:** Provide graphics for maze walls and other players, consistent with a "Maze Wars" aesthetic.
*   **Performance Display:** Show the current frame rate.
*   **Score Display:** Display player scores alongside the mini-map, including a dynamic leaderboard.
*   **Lobby Chat:** Implement a mechanism for players to send and receive text messages in the lobby.
*   **Connection Process:**
    *   Prompt clients to enter the IP address and port of the game server.
    *   Prompt clients to provide a username for identification.
*   **Enhanced GUI Initialization:**
    *   Manage game initialization.
    *   Save a history of previously connected servers with user-defined aliases for simplified reconnection.
*   **Level Editor:** Implement a user-friendly level editor for players to design and create custom mazes.

**Game Environments and Content:**

*   **Distinct Game Levels:** Incorporate at least 3 distinct game levels with increasingly complex maze designs (e.g., higher density of dead ends).
*   **Dynamic Maze Generation:** Implement dynamic maze creation based on difficulty and the number of players.

**Game Creation:** Host creates a game on the server with settings and stars it.
**Performance Targets:** Maintain stable 50 FPS.



## Features

### Application Modes
- **Host Game Mode**:
Starts a game server and automatically connects the local client to it
- **Join Game Mode**:
Connects the local client to an existing game server

### Core Gameplay
- **Maze Navigation**:
Players move through a flat, grid-based maze environment
- **Projectile Combat**:
Players can shoot projectiles at opponents
- **Hit Scoring**:
Awards 10 points for hitting an opponent
- **Damage Scoring**:
Deducts 5 points for being hit by an opponent
- **Post-Hit Grace Period**:
Players are invulnerable for 2 seconds after being hit
- **Winning Condition**:
Game ends when a player achieves a predefined score
- **Dynamic Leaderboard**:
Displays real-time player scores and rankings


### Networking & Connectivity
- **Multiplayer Support**:
Allows 10 to 30 concurrent players in a game
- **Cross-Machine Connectivity**:
Clients can connect to servers running on different physical machines

### User Interface & Experience
- **Mini-Map Display**:
Shows the player's current position and the overall maze layout
- **Game World Graphics**:
Visual representation of maze walls and other players
- **FPS Display**:
Shows the current frames per second
- **Player Score Display**:
Shows the individual player's current score
- **Lobby Chat**:
Allows players to send and receive text messages in the lobby
- **Server Connection Input**:
Prompts clients to enter the server's IP address and port
- **Username Input**:
Prompts clients to provide a username for identification
- **Server Connection History**:
Saves previously connected server details with user-defined aliases
- **Simplified Reconnection**:
Allows quick reconnection to saved servers (displays up to 5 saved servers)

### Game Content & Environment
- **Multiple Pre-designed Levels**:
Provides at least 3 distinct maze levels with varying complexity
- **Dynamic Maze Generation**:
Creates mazes procedurally based on difficulty and player count
- **Custom Maze Editor**:
Allows users to design and create custom maze levels

### Game Management
- **Game Hosting**:
Allows a host to create a new game session on the server
- **Game Settings Configuration**:
Host can define game parameters (e.g., winning score, maze difficulty)
- **Game Start Initiation**:
Host can initiate the start of a created game

## Tech Stack
- Tech stack
    *   **Core Development**
        *   `language/runtime: Rust`
        *   `build_tool/package_manager: cargo`
    *   **Networking & Communication**
        *   `networking/protocol: UDP`
        *   `networking/implementation: std::net::UdpSocket (non-blocking, manual polling)`
        *   `data/serialization: bincode`
    *   **Graphics & User Interface**
        *   `graphics/windowing: winit`
        *   `graphics/rendering_backend: pixels`
        *   `graphics/math: glam`
    *   **Audio**
        *   `audio/playback: rodio`
    *   **Application Logic & Abstraction (Custom Crates)**
        *   `fps_ui`: UI framework built on winit and pixels, providing components, layout system, and event handling
        *   `fps_net`: UDP networking library with reliable/unreliable messaging, ping management, and client/server sockets
        *   `fps_levels`: Maze generation and management, including procedural generation and configuration
        *   `fps_config`: Configuration management for user settings and preferences
        *   `fps_audio`: Audio playback abstraction (Not implemented)

- High-level setup
    *   **Single Executable, Dual Mode:** The application compiles into a single binary. Upon launch, the user is presented with a UI to choose between "Host Game" (starts a local server and connects as a client), "Join Game" (connects as a client to an existing server), or "Level Editor".
    *   **Game Logic (in `src/app/`):**
        *   Game state management, physics, scoring, and player actions are handled in the main application code
        *   Server acts as authoritative source for game state
        *   Client performs local prediction and rendering
        
    *   **Client-Side Architecture:**
        *   Main thread handles UI rendering and user input via `fps_ui` crate
        *   Separate networking thread (`game_client_net.rs`) handles UDP communication
        *   Game rendering uses raycasting engine (`GameRender` component) for 3D visualization
        *   UI components display mini-map, scores, FPS counter, and chat
        
    *   **Server-Side Architecture:**
        *   Single-loop design for:
            *   **Networking:** Receives client inputs via UDP, sends game state snapshots
            *   **Game logic:** Updates authoritative game state at 60 Hz tick rate
        *   Manages client connections, disconnections, and game progression
        *   Handles scoring, invincibility periods, and winning conditions

    *   **User Interface (`fps_ui` crate):**
        *   Component-based UI system with layers and layout management
        *   Views for main menu, join/host game, lobby, and in-game HUD
        *   Components include buttons, labels, text inputs, mini-map, game renderer, and more
        *   Processes keyboard and mouse input for both UI and game control

    *   **Level/Maze System (`fps_levels` crate):**
        *   Procedural maze generation using DFS algorithm
        *   Configurable maze sizes and difficulties
        *   Maze editor component for custom level creation
        *   Support for saving and loading custom mazes

    *   **Server Tick Rate:**
        *   Server game loop runs at 60 Hz (16.67 ms per tick)
        *   Processes client inputs and updates game state
        *   Broadcasts game snapshots to all connected clients
        
    *   **Client Rendering:**
        *   Client renders at display refresh rate (targeting 60+ FPS)
        *   Uses raycasting for 3D first-person view
        *   Displays mini-map, leaderboard, FPS counter, and crosshair
        *   Handles player and bullet rendering with depth sorting

## Project Structure
```
multiplayer-fps/
├── src/
│   ├── main.rs                    # Application entry point
│   ├── app/                       # Core application logic
│   │   ├── mod.rs                 # Module exports
│   │   ├── app.rs                 # Main App struct and event loop
│   │   ├── client.rs              # Client state management
│   │   ├── constants.rs           # Game constants (physics, scoring, etc.)
│   │   ├── game.rs                # Game logic and state updates
│   │   ├── game_client.rs         # Client-side game handling
│   │   ├── game_client_net.rs     # Client networking thread
│   │   ├── game_server.rs         # Server-side game handling
│   │   ├── game_structs.rs        # Game state structures
│   │   ├── game_input.rs          # Input state management
│   │   ├── pos.rs                 # Position and rotation utilities
│   │   └── view_ids.rs            # View and component ID constants
│   ├── view/                      # UI Views (screens)
│   │   ├── mod.rs                 # View module exports
│   │   ├── view.rs                # View trait definition
│   │   ├── view_main_menu.rs      # Main menu screen
│   │   ├── view_join_menu.rs      # Join game screen
│   │   ├── view_host_menu.rs      # Host game screen
│   │   ├── view_level_menu.rs     # Level selection screen
│   │   ├── view_lobby.rs          # Lobby screen
│   │   └── view_game.rs           # In-game screen
│   └── assets/                    # Game assets
│       ├── fonts/                 # Font files
│       ├── image/                 # Texture images
│       └── config.json            # Configuration file
├── crates/                        # Shared library crates
│   ├── fps_ui/                    # UI framework crate
│   │   ├── src/
│   │   │   ├── lib.rs             # UI crate exports
│   │   │   ├── components/        # UI components
│   │   │   │   ├── mod.rs
│   │   │   │   ├── component.rs   # Component trait
│   │   │   │   ├── button.rs      # Button component
│   │   │   │   ├── label.rs       # Label component
│   │   │   │   ├── text_input.rs  # Text input component
│   │   │   │   ├── text_box.rs    # Multi-line text box
│   │   │   │   ├── panel.rs       # Panel/background component
│   │   │   │   ├── fps_counter.rs # FPS display component
│   │   │   │   ├── game_render.rs # 3D game renderer (raycasting)
│   │   │   │   ├── mini_map.rs    # Mini-map component
│   │   │   │   ├── maze_viewer.rs # Maze display component
│   │   │   │   └── maze_editor.rs # Maze editor component
│   │   │   ├── context.rs         # UI context and resources
│   │   │   ├── driver.rs          # Window and rendering driver
│   │   │   ├── events.rs          # UI events and updates
│   │   │   ├── fonts.rs           # Font management
│   │   │   ├── geometry.rs        # Geometric primitives
│   │   │   ├── layout.rs          # Layout system
│   │   │   └── manager.rs         # UI manager and layers
│   │   └── README.md              # UI crate documentation
│   ├── fps_net/                   # Networking crate
│   │   ├── src/
│   │   │   ├── lib.rs
│   │   │   ├── protocol.rs        # Message protocol definitions
│   │   │   ├── message.rs         # Message serialization
│   │   │   ├── net_socket.rs      # Base UDP socket wrapper
│   │   │   ├── client_socket.rs   # Client-side socket
│   │   │   ├── server_socket.rs   # Server-side socket
│   │   │   ├── ping.rs            # Ping/pong management
│   │   │   └── util.rs            # Network utilities
│   │   └── README.md              # Network crate documentation
│   ├── fps_levels/                # Level/maze generation crate
│   │   ├── src/
│   │   │   ├── lib.rs
│   │   │   ├── config.rs          # Maze configuration
│   │   │   ├── generator.rs        # Maze generation algorithms
│   │   │   └── maze.rs            # Maze data structures
│   │   └── README.md              # Levels crate documentation
│   ├── fps_config/                 # Configuration crate
│   │   └── src/
│   │       └── lib.rs
│   └── fps_audio/                  # Audio crate (placeholder)
│       └── src/
│           └── main.rs
├── Cargo.toml                     # Workspace and main crate config
└── README.md                       # This file
```

## Implementation Status

### ✅ Completed Features

#### Core Gameplay
- ✅ Game view and rendering system
- ✅ Player movement (WASD + Shift for running)
- ✅ Mouse look (camera rotation)
- ✅ Shooting mechanics
- ✅ Bullet system and rendering
- ✅ Player rendering with 3D sprites
- ✅ Hit detection and scoring system
- ✅ Invincibility grace period (2 seconds)
- ✅ Winning condition based on target score
- ✅ Dynamic leaderboard display

#### Networking
- ✅ UDP client-server architecture
- ✅ Reliable and unreliable message delivery
- ✅ Ping/pong system for connection health
- ✅ Client input transmission
- ✅ Game state snapshot broadcasting
- ✅ Client connection/disconnection handling

#### User Interface
- ✅ Main menu view
- ✅ Host game view
- ✅ Join game view with saved server history
- ✅ Level selection view
- ✅ Lobby view with chat
- ✅ Server connection history with aliases
- ✅ In-game HUD with:
    - ✅ 3D raycasted game view
    - ✅ Mini-map with player position
    - ✅ Leaderboard
    - ✅ FPS counter with background
    - ✅ Crosshair
    - ✅ Score display

#### Game World
- ✅ Procedural maze generation
- ✅ Multiple maze sizes and difficulties
- ✅ Maze rendering (walls, floor, ceiling)
- ✅ Collision detection
- ✅ Player positioning and movement
- ✅ Custom maze editor
- ✅ Save/load custom mazes

#### Code Quality
- ✅ Centralized constants module
- ✅ View and component ID constants
- ✅ Comprehensive documentation for all crates
- ✅ Refactored game logic into helper functions
- ✅ Component-based UI architecture

### 🚧 In Progress / Planned

#### Audio
- [ ] Sound effects
- [ ] Background music
- [ ] Audio manager integration

### 📝 Notes
- Server runs at 60 Hz tick rate
- Client renders at display refresh rate
- Uses DDA (Digital Differential Analyzer) raycasting for 3D rendering
- All game constants are centralized in `src/app/constants.rs`

