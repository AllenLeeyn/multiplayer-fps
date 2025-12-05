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
*   **Distinct Player Roles:** Two player types, "Spheres" and "Cubes," each with unique abilities that encourage different playstyles and tactical depth.
    *    Spheres: Possess the ability to shrink and move rapidly, facilitating evasion and strategic hiding.
    *    Cubes: Can mimic maze walls, allowing them to blend in and ambush opponents.

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
**AI Opponents:** Develop AI-controlled players for gameplay when human players are unavailable.
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

### Player Roles & Abilities
- **Player Role Selection**:
Players choose between Sphere and Cube roles
- **Sphere Ability (Shrink & Speed)**:
Allows players to shrink and move rapidly for evasion and hiding
- **Cube Ability (Mimic Wall)**:
Allows players to blend in by mimicking maze walls

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
Allows quick reconnection to saved servers

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
- **AI Opponent Integration**:
Computer-controlled players can participate in games

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
        *   `graphics/math: gaml`
    *   **Audio**
        *   `audio/playback: rodio`
    *   **Application Logic & Abstraction (Custom Crates)**
        *   `shared_logic: core` (Defines shared game entities, rules, physics, and authoritative game state.)
        *   `network_abstraction: net` (Manages UDP packet handling, client connection tracking, and message serialization/deserialization.)
        *   `rendering_engine: render` (Handles 3D rendering pipeline, translating game state into visual output.)
        *   `user_interface_components: ui` (Manages all graphical user interface elements and processes user input.)
        *   `game_content_management: levels` (Provides functionality for defining, loading, and generating maze layouts.)
        *   `audio_manager: audio` (Abstracts audio playback for sound effects and music.)

- High-level setup
    *   **Single Executable, Dual Mode:** The application compiles into a single binary. Upon launch, the user is presented with a UI to choose between "Host Game" (starts a local server and connects as a client), "Join Game" (connects as a client to an existing server), or "Level Editor".
    *   **Shared Game Logic (`core` crate):**
        *   The `core` crate is fundamental, containing all game rules, physics, entity definitions, scoring logic, and the authoritative game state.
        *   It is used by both the server (for authoritative state updates) and the client (for local prediction, interpolation, and rendering).
        
    *   **Client-Side Setup:**
        *   Manages user input (`ui` crate), renders the game world (`render` crate), displays UI elements (mini-map, scores, FPS, chat via `ui` crate), and handles audio (`audio` crate).
        *   Communicates with the server via the `net` crate, sending player inputs and receiving game state updates.
        *   Performs client-side prediction and interpolation using the `core` crate to provide a smooth experience despite network latency.

    *   **Server-Side Setup (when hosting):**
        *   Operates as the authoritative source for the game state.
        *   Employs a multi-threaded design for optimal performance and responsiveness:
            *   **UDP Read Thread:** Continuously receives incoming UDP packets (client inputs) via `std::net::UdpSocket` and `net` crate, deserializes them (`bincode`), and queues them for processing.
            *   **Gameplay Loop Thread:** The main game logic thread. It dequeues client inputs, updates the authoritative game state using the `core` crate (handling physics, scoring, AI, player abilities, grace periods), and prepares game state updates for clients. It also manages game progression (winning conditions, level changes).
            *   **UDP Write Thread:** Sends outgoing UDP packets (game state updates) to all connected clients via `std::net::UdpSocket` and `net` crate, serializing data (`bincode`).
        *   Manages client connections, disconnections, and authentication (username).
        *   Integrates AI opponents using the `core` crate's game logic.

    *   **User Interface (`ui` crate):**
        *   Handles all GUI elements, including main menu, server connection prompts, lobby chat, in-game HUD (mini-map, scores, FPS), and the level editor.
        *   Processes user input (keyboard, mouse) for both game control and UI interaction.

    *   **Level Editor (`levels` & `ui` crates):**
        *   A dedicated mode within the application allowing users to design custom mazes.
        *   Leverages the `ui` crate for editor controls and the `levels` crate for maze data representation and saving/loading.

    *   Server-Side (30 FPS Game Logic):
        *   The server's "Gameplay Loop Thread" will be configured to run at a fixed tick rate of 30 Hz (e.g., every 33.3 ms).
        *   Within each server tick, the core crate will be used to:
            *   Process all client inputs received since the last tick.
            *   Update the authoritative game state (physics, scoring, AI, player abilities, grace periods).
            *   Generate a new authoritative game state snapshot.
        *   The net crate on the server will send these game state snapshots to all connected clients at the 30 Hz server tick rate.
        *   This reduces the computational load on the server, allowing it to handle more concurrent players or run on less powerful hardware.
        
    *   Client-Side (60 FPS Rendering & Prediction):
        *   The client's main loop will run as fast as possible, aiming for 60 FPS (or higher, capped by winit and pixels).
        *   Input Handling: The client will process user input (via ui crate) at its full frame rate (60 Hz). These inputs will be immediately used for client-side prediction (using the core crate) to provide instant feedback to the player.
        *   Input Sending: The net crate on the client will send player inputs to the server. Instead of debouncing to 1/30 sec, it's generally better to send inputs more frequently (e.g., every frame at 60 Hz) or batch them and send them at a rate slightly higher than the server tick rate (e.g., 45-60 Hz). This ensures inputs arrive promptly without introducing artificial lag.
        *   State Updates: The client's net crate will receive game state snapshots from the server at 30 Hz.
        *   Prediction & Interpolation: The client's core crate will use these server snapshots, combined with client-side prediction and interpolation techniques, to smoothly render the game world at 60 FPS (via render crate) and update the UI (via ui crate). This involves predicting future states based on local input and inte

## Project Structure
```
multiplayer_fps/
├── src/
│   ├── application/
│   └── modes/
│       ├── host_game/
│       ├── join_game/
│       └── level_editor/
├── components/
│   ├── fps_core/
│   ├── fps_net/
│   │   ├── src/
│   │   │   ├── protocol.rs
│   │   │   │   •   pub enum MessageType {
│   │   │   │           Ping=0,
│   │   │   │           Pong=1,
│   │   │   │           ConnectRequest=2,
│   │   │   │           ConnectAccept=3,
│   │   │   │           ConnectDeny=4,
│   │   │   │           DisconnectNotice=5,
│   │   │   │           Reliable=6,
│   │   │   │           Unreliable=7,
│   │   │   │           Acknowledgement=8,
│   │   │   │       }
│   │   │   │   •   pub struct MessageHeader {
│   │   │   │           pub msg_type: MessageType,
│   │   │   │           pub sequence: u32,
│   │   │   │           pub timestamp_ms: u64,
│   │   │   │       }
│   │   │   │
│   │   │   ├── message.rs
│   │   │   │   •   pub struct Message {
│   │   │   │           pub header: MessageHeader,
│   │   │   │           pub payload: Option<Vec<u8>>,
│   │   │   │       }
│   │   │   │   •   impl Message {
│   │   │   │           pub fn new(msg_type: MessageType, sequence: u32, payload: Option<Vec<u8>>) -> Self
│   │   │   │           pub fn encode(&self) -> Result<Vec<u8>, Box<dyn Error>>
│   │   │   │           pub fn decode(bytes: &[u8]) -> Result<Self, Box<dyn Error>>
│   │   │   │           pub fn decode_payload<T: for<'de> Deserialize<'de>>(&self) -> Result<T, Box<dyn Error>>
│   │   │   │           pub fn new_ping(sequence: u32) -> Self
│   │   │   │           pub fn new_pong(sequence: u32) -> Self
│   │   │   │           pub fn new_connect_request(sequence: u32, username: &str) -> Self
│   │   │   │           pub fn new_connect_accept(sequence: u32) -> Self
│   │   │   │           pub fn new_connect_deny(sequence: u32, reason: &str) -> Self
│   │   │   │           pub fn new_disconnect_notice(sequence: u32) -> Self
│   │   │   │           pub fn new_reliable<T: Serialize>(sequence: u32, payload_obj: &T) -> Self
│   │   │   │           pub fn new_unreliable<T: Serialize>(sequence: u32, payload_obj: &T) -> Self
│   │   │   │           pub fn new_ack(sequence: u32, acked_sequence: u32) -> Self
│   │   │   │           pub fn is_ping(&self) -> bool
│   │   │   │           pub fn is_pong(&self) -> bool
│   │   │   │           pub fn is_reliable(&self) -> bool
│   │   │   │           pub fn is_unreliable(&self) -> bool
│   │   │   │           pub fn is_ack(&self) -> bool
│   │   │   │           pub fn is_connect_request(&self) -> bool
│   │   │   │           pub fn is_connect_accept(&self) -> bool
│   │   │   │           pub fn is_connect_deny(&self) -> bool
│   │   │   │           pub fn is_disconnect_notice(&self) -> bool
│   │   │   │       }
│   │   │   │
│   │   │   ├── ping.rs
│   │   │   │   •   struct PingEntry {
│   │   │   │           timestamp: Instant,
│   │   │   │           addr: SocketAddr,
│   │   │   │       }
│   │   │   │   •   pub struct PingManager {
│   │   │   │           next_sequence: u32,
│   │   │   │           pending_pings: HashMap<u32, PingEntry>,
│   │   │   │           timeout: Duration,
│   │   │   │       }
│   │   │   │   •   impl MePingManagerssage {
│   │   │   │           pub fn new(timeout: Duration) -> Self
│   │   │   │           pub fn create_ping(&mut self) -> u32
│   │   │   │           pub fn handle_pong(&mut self, sequence: u32) -> Option<Duration>
│   │   │   │           pub fn check_timeouts(&mut self) -> Vec<SocketAddr> 
│   │   │   │       }
│   │   │   │
│   │   │   ├── net_socket.rs
│   │   │   │   •   struct ReliableEntry {
│   │   │   │           msg: Message,
│   │   │   │           addr: SocketAddr,
│   │   │   │           last_sent: Instant,
│   │   │   │           attempts: u32,
│   │   │   │       }
│   │   │   │   •   pub struct NetSocket {
│   │   │   │           recv_socket: UdpSocket,
│   │   │   │           send_socket: UdpSocket,
│   │   │   │           sequence: u32,
│   │   │   │           recv_buffer: Vec<u8>,
│   │   │   │           pub timeout: Duration,
│   │   │   │           reliable_queue: HashMap<u32, ReliableEntry>,
│   │   │   │       }
│   │   │   │   •   impl NetSocket {
│   │   │   │           pub fn bind(addr: &str, timeout: Duration) -> Result<Self>
│   │   │   │           pub fn recv(&mut self) -> Result<(Message, SocketAddr)>
│   │   │   │           pub fn next_sequence(&mut self) -> u32
│   │   │   │           pub fn send(&mut self, addr: SocketAddr, msg: &Message) -> Result<usize>
│   │   │   │           pub fn send_reliable(&mut self, addr: SocketAddr, msg: &Message) -> Result<usize>
│   │   │   │           pub fn resend_pending(&mut self) -> Result<()>
│   │   │   │           pub fn set_timeout(&self, timeout: Duration) -> Result<()>
│   │   │   │           pub fn local_addr(&self) -> Result<SocketAddr>
│   │   │   │       }
│   │   │   │
│   │   │   ├── client_socket.rs
│   │   │   │   •   pub struct ClientSocket {
│   │   │   │           socket: NetSocket,
│   │   │   │           server_addr: SocketAddr,
│   │   │   │           ping_manager: PingManager,
│   │   │   │       }
│   │   │   │   •   impl ClientSocket {
│   │   │   │           pub fn new(local_addr: &str, server_addr: &str, ping_timeout: Duration) -> Result<Self>
│   │   │   │           pub fn recv(&mut self) -> Result<Option<Message>>
│   │   │   │           pub fn next_sequence(&mut self) -> u32
│   │   │   │           pub fn send(&mut self, msg: &Message) -> Result<usize> 
│   │   │   │           pub fn send_reliable(&mut self, msg: &Message) -> Result<usize>
│   │   │   │           pub fn resend_pending(&mut self) -> Result<()>
│   │   │   │           pub fn send_ping(&mut self) -> Result<u32>
│   │   │   │           pub fn handle_pong(&mut self, seq: u32) -> Option<Duration>
│   │   │   │           pub fn handle_ping(&self, seq: u32) -> Result<Message>
│   │   │   │           pub fn check_ping_timeouts(&mut self) -> Vec<u32>
│   │   │   │           pub fn local_addr(&self) -> Result<SocketAddr>
│   │   │   │       }
│   │   │   │
│   │   │   ├── server_socket.rs
│   │   │   │   •   pub struct ClientInfo {
│   │   │   │           pub last_seen: u64,
│   │   │   │       }
│   │   │   │   •   pub struct ServerSocket {
│   │   │   │           socket: Arc<Mutex<NetSocket>>,
│   │   │   │           clients: Arc<RwLock<HashMap<SocketAddr, ClientInfo>>>,
│   │   │   │           ping_manager: Arc<Mutex<PingManager>>,
│   │   │   │           timeout: Duration,
│   │   │   │       }
│   │   │   │   •   impl ServerSocket {
│   │   │   │           pub fn bind(local_addr: &str, timeout: Duration) -> Result<Self>
│   │   │   │           pub fn recv(&mut self) -> Result<(Message, SocketAddr)> 
│   │   │   │           pub fn next_sequence(&mut self) -> u32
│   │   │   │           pub fn send(&mut self, addr: SocketAddr, msg: &Message) -> Result<usize>
│   │   │   │           pub fn send_reliable(&mut self, addr: SocketAddr, msg: &Message) -> Result<usize>
│   │   │   │           pub fn resend_pending(&mut self) -> Result<()>
│   │   │   │           pub fn broadcast(&mut self, msg: &Message) -> Result<Vec<SocketAddr>>
│   │   │   │           pub fn send_ping(&mut self, addr: SocketAddr) -> Result<u32>
│   │   │   │           pub fn handle_pong(&mut self, addr: SocketAddr, seq: u32) -> Option<Duration>
│   │   │   │           pub fn handle_ping(&self, addr: SocketAddr, seq: u32) -> Result<(SocketAddr, Message)>
│   │   │   │           pub fn check_ping_timeouts(&mut self) -> Vec<u32>
│   │   │   │           pub fn client_list(&self) -> Vec<SocketAddr>
│   │   │   │           pub fn remove_stale_clients(&mut self) -> Vec<SocketAddr>
│   │   │   │           pub fn local_addr(&self) -> Result<SocketAddr>
│   │   │   │       }
│   │   │   │
│   │   │   ├── lib.rs
│   │   │   ├── error.rs
│   │   │   └── util.rs
│   │   │       •   pub fn now_ms() -> u64
│   │   │
│   │   └── Cargo.toml
│   ├── fps_render/
│   ├── fps_ui/
│   ├── fps_levels/
│   └── fps_audio/
├── assets/
│   ├── textures/
│   ├── sounds/
│   ├── fonts/
│   └── maps/
├── config/
├── .gitignore
└── README.md
```
