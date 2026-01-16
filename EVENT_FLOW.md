# Event Flow Architecture

This document explains the relationship between `UIEvent`, `ViewAction`, `GameNetCommand`, and `GameNetEvent` and how they flow through the application architecture.

## Overview

The application uses a layered event system to handle user interactions and network communication:

```
UI Components → UIEvent → View → ViewAction → App → GameClient → GameNetCommand
                                                                        ↓
Network Thread ← GameNetEvent ← GameClient ← ComponentUpdate ← App ← ViewAction
```

## Type Definitions

### 1. `UIEvent` (Low-Level Component Events)

**Location:** `crates/fps_ui/src/events.rs`

**Purpose:** Represents low-level user interactions detected by UI components.

**Flow:** Components → UIManager → App → Views

**Variants:**
- `ButtonClicked(String)` - Button component ID
- `TextChanged(String, String)` - Component ID and new text
- `TextSubmitted(String, String)` - Component ID and submitted text
- `ButtonToggled(String, bool)` - Component ID and toggle state
- `ValueChanged(String, f32)` - Component ID and numeric value
- `ExitRequested` - User requested exit

**Example:**
```rust
// User clicks a button
UIEvent::ButtonClicked("join_button".to_string())
```

---

### 2. `ViewAction` (High-Level Application Actions)

**Location:** `src/view/view.rs`

**Purpose:** Represents application-level actions that views produce from UIEvents. These are domain-specific actions rather than component-specific events.

**Flow:** Views → App → Game Logic

**Variants:**
- `SwitchTo(String)` - Switch to another view
- `QuitApp` - Exit application
- `UpdateComponent(Vec<ComponentUpdate>)` - Update component states
- `SaveUsername(String)` - Save username to config
- `HostGame { game_name, maze, target_score }` - Start hosting a game
- `JoinGame(String)` - Connect to a game server
- `LeaveLobby` - Disconnect from current game
- `SendChatMessage(String)` - Send chat message
- `SendStartGame` - Request game start (host only)
- `StartGame` - Transition to game view
- `GameEnd(String)` - Handle game end with winner
- `PlayerKeyboardInput(HashSet<String>)` - Player movement input
- `PlayerMouseInput(bool)` - Player shooting input

**Example:**
```rust
// View converts button click to action
UIEvent::ButtonClicked("join_button") 
  → ViewAction::SwitchTo("join_menu".to_string())
```

---

### 3. `GameNetCommand` (Commands to Networking Thread)

**Location:** `src/app/game_client_net.rs`

**Purpose:** Commands sent FROM the game logic TO the networking thread to request network operations.

**Flow:** GameClient → Networking Thread

**Variants:**
- `Send(Message)` - Send unreliable message
- `SendReliable(Message)` - Send reliable message (with retry)
- `Shutdown` - Shutdown networking thread

**Example:**
```rust
// App wants to send chat message
ViewAction::SendChatMessage("Hello!") 
  → GameClient::send_chat_msg() 
  → GameNetCommand::SendReliable(chat_message)
```

---

### 4. `GameNetEvent` (Events from Networking Thread)

**Location:** `src/app/game_client_net.rs`

**Purpose:** Events sent FROM the networking thread TO the game logic when network messages are received.

**Flow:** Networking Thread → GameClient → App → Views/Components

**Variants:**
- `ClientList(Vec<String>)` - Updated list of connected clients
- `GameStart` - Game has started
- `GameEnd(String)` - Game ended with winner
- `Snapshot(Message)` - Game state snapshot from server
- `Chat(Message)` - Chat message received
- `Disconnected` - Connection lost

**Example:**
```rust
// Server sends chat message
Server → Network Thread → GameNetEvent::Chat(msg)
  → GameClient::poll() 
  → ComponentUpdate::AppendText(...)
```

---

## Complete Flow Examples

### Example 1: User Clicks "Join Game" Button

```
1. User clicks "join_button" in main menu
   └─> UIEvent::ButtonClicked("join_button")

2. Main menu view handles the event
   └─> View::handle_ui_events(UIEvent)
   └─> ViewAction::SwitchTo("join_menu")

3. App processes the ViewAction
   └─> App::handle_view_action(ViewAction::SwitchTo)
   └─> App::activate_view("join_menu")
   └─> Updates UI to show join menu
```

### Example 2: User Joins a Game Server

```
1. User enters server address and clicks "Connect"
   └─> UIEvent::ButtonClicked("connect_button")

2. Join menu view handles the event
   └─> ViewAction::JoinGame("192.168.1.100:9000")

3. App processes the action
   └─> App::handle_view_action(ViewAction::JoinGame)
   └─> App::join_game(server_addr)
   └─> Creates GameClient and networking thread
   └─> Sends connection request via GameNetCommand::SendReliable

4. Networking thread processes command
   └─> Sends Message via UDP socket

5. Server responds with connection acceptance
   └─> Network thread receives Message
   └─> GameNetEvent::ClientList(users)

6. GameClient polls events
   └─> GameClient::poll() receives GameNetEvent
   └─> Produces ComponentUpdate::SetTextVec(...)
   └─> Updates lobby user list component
```

### Example 3: Chat Message Flow

```
1. User types message and presses Enter
   └─> UIEvent::TextSubmitted("lobby_chat_input", "Hello!")

2. Lobby view handles the event
   └─> ViewAction::SendChatMessage("Hello!")

3. App processes the action
   └─> App::handle_view_action(ViewAction::SendChatMessage)
   └─> GameClient::send_chat_msg()
   └─> GameNetCommand::SendReliable(chat_message)

4. Networking thread sends message
   └─> UDP socket sends to server

5. Server broadcasts to all clients (including sender)
   └─> Server receives message
   └─> Server broadcasts MessageType::ChatMessage

6. Client's networking thread receives broadcast
   └─> GameNetEvent::Chat(msg)

7. GameClient polls and processes
   └─> GameClient::poll() matches (Lobby, Chat)
   └─> ComponentUpdate::AppendText("username: Hello!\n")
   └─> App applies update to chat log component
```

### Example 4: In-Game State Updates

```
1. Server sends game state snapshot (60 Hz)
   └─> Server broadcasts MessageType::GameSnapShot

2. Client networking thread receives
   └─> GameNetEvent::Snapshot(msg)

3. GameClient polls event
   └─> GameClient::poll() matches (InGame, Snapshot)
   └─> Decodes snapshot payload
   └─> Updates internal game state
   └─> Produces multiple ComponentUpdates:
       - ComponentUpdate::SetGameRender(...)  // Update players/bullets
       - ComponentUpdate::SetMiniMapPlayer(...)  // Update minimap
       - ComponentUpdate::SetTextVec(...)  // Update leaderboard

4. App applies updates
   └─> App::manager.apply_updates(updates)
   └─> UI components re-render with new data
```

---

## Key Architectural Decisions

### Separation of Concerns

1. **UIEvent** - Pure UI layer, no domain knowledge
   - Components only know about UI interactions
   - No game-specific logic in components

2. **ViewAction** - Domain-specific actions
   - Views translate UI events to application actions
   - Views know about game domain but not networking details

3. **GameNetCommand/Event** - Networking abstraction
   - Game logic doesn't know about UDP sockets
   - Network thread handles all socket operations
   - Clean separation between game logic and networking

### Thread Safety

- **Main Thread:** UI rendering, event processing, game logic
- **Networking Thread:** UDP socket I/O, message encoding/decoding
- **Communication:** `mpsc::channel` for thread-safe message passing
  - `GameNetCommand` channel: Main thread → Network thread
  - `GameNetEvent` channel: Network thread → Main thread

### Event Processing Patterns

1. **UIEvent → ViewAction:** Synchronous, in main thread
   - Views immediately process UI events
   - Views produce ViewActions

2. **ViewAction → GameNetCommand:** Synchronous, in main thread
   - App processes ViewActions immediately
   - App sends commands to network thread via channel

3. **Network → GameNetEvent:** Asynchronous, polled
   - Network thread receives messages asynchronously
   - Main thread polls events via `try_recv()` each frame
   - Non-blocking to maintain frame rate

---

## Benefits of This Architecture

1. **Loose Coupling:** Each layer only knows about adjacent layers
   - Components don't know about Views
   - Views don't know about networking
   - Game logic doesn't know about UI components

2. **Testability:** Each layer can be tested independently
   - Mock UIEvents to test Views
   - Mock GameNetEvents to test game logic
   - Mock ViewActions to test App

3. **Maintainability:** Clear responsibility boundaries
   - UI changes don't affect networking code
   - Protocol changes don't affect UI components
   - Game logic changes don't affect view rendering

4. **Flexibility:** Easy to add new features
   - New UI component? Add new UIEvent variant
   - New game action? Add new ViewAction variant
   - New network message? Add new GameNetEvent variant
