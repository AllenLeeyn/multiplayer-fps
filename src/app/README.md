# Application Layer (`src/app`)

Core application logic for the multiplayer FPS: coordinates views, game state, and networking between the UI (views) and the network layer. All event handling goes through winit’s **ApplicationHandler**; the app never polls for events.

---

## 1. Architecture and modules

### High-level flow

```
UI (fps_ui)  →  View (src/view)  →  App (app.rs)  →  Game logic (game_client, game_server, game)
     WindowEvent         ViewAction      GameNetCommand/Event         UDP (fps_net)
```

- **Views** turn UI events into `ViewAction`s (switch view, host/join, send chat, etc.).
- **App** handles `ViewAction`s, owns `UIManager`, and holds optional `ServerHandle` and `Game` (client). It does not run a manual event loop; winit drives it via `ApplicationHandler`.
- **Game logic**: `game.rs` (shared movement/bullet/collision; server-authoritative), `game_server.rs` (server loop), `game_client.rs` (client state + snapshot interpolation).
- **Networking**: server and client each use a dedicated thread; they communicate with App via channels.

### Module layout

| File | Purpose |
|------|--------|
| **`app.rs`** | Main `App`; implements `ApplicationHandler`. Owns driver, `UIManager`, views, config, optional server/game. Dispatches `ViewAction`, drives redraw and input. |
| **`client.rs`** | Server-side client: `Client`, `ClientList`, `ClientStatus` (Normal/Invincible). |
| **`constants.rs`** | Tick rate, physics sizes, player/bullet params, timeouts, intervals. |
| **`game.rs`** | Shared logic: spawn, invincibility, `handle_player_movement`, bullet step and collision. Used by server; client displays interpolated snapshots. |
| **`game_client.rs`** | Client-side `Game`: connection, local players/bullets/maze, `GameNetHandle`, interpolation. `connect()`, `poll()`, `send_game_input()`, `send_chat_msg()`, `send_game_start()`, etc. |
| **`game_client_net.rs`** | Client network thread: `GameNetCommand` / `GameNetEvent`, `GameNetHandle`. |
| **`game_server.rs`** | Authoritative server: `GameServer`, `ServerHandle`. Thread runs `run(cmd_rx)`; handles joins, input, tick loop, snapshots, chat. |
| **`game_structs.rs`** | Shared: `GameState`, `PlayerAction`, `Bullet`, `BulletStatus`. |
| **`game_input.rs`** | `GameInputState`: keyboard/mouse → actions + mouse_dx; `handle_game_input()`, `handle_mouse_motion()`, `snapshot()` → `GameInputPayload`. |
| **`pos.rs`** | `Pos`: x, y, angle; movement and rotation helpers. |
| **`view_ids.rs`** | View and component ID constants (no magic strings). |

---

## 2. Implementation overview

### Startup (from `main.rs`)

1. Load **Config** from `config_path` (e.g. `src/assets/config.json`).
2. Create **UIMainContext** (font, logical size), load textures, then **UIManager** and global layers.
3. Build **App** with `driver: None`, manager, views, config, `server: None`, `game: None`, `game_input`, `last_input_send: None`.
4. Register each view (`ViewMainMenu`, `ViewJoinMenu`, `ViewHostMenu`, `ViewLevelMenu`, `ViewLobby`, `ViewGame`) and **activate** main menu.
5. Run **`event_loop.run_app(&mut app)`**; winit owns the loop and calls `ApplicationHandler` methods.

The **window is created in `ApplicationHandler::resumed`**, not in `main`, so we have a single code path and correct behavior on suspend/resume (e.g. mobile). If `AppDriver::new` fails there, we call `event_loop.exit()`.

### Winit: ApplicationHandler only

We use winit **only** through the **ApplicationHandler** trait: no `run()` with a closure or manual `poll`/`next`. Flow:

1. `EventLoop::new()` and build `App`.
2. `event_loop.run_app(&mut app)`.
3. Winit invokes: `resumed`, `device_event`, `window_event`, `about_to_wait`.

**Why:** Single ownership of the loop, cross-platform lifecycle (window in `resumed`), and clear callbacks per event type.

**Logical vs physical dimensions:** UI is laid out in **logical** coordinates (e.g. 800×450); the window has **physical** pixels. The driver converts **cursor position** from physical to logical in `handle_winit_event` so hit-testing and hover use logical coords. Scale factor is shared with resize so behavior is consistent across resolution and DPI.

**Mouse for look:** We use **`device_event`** for mouse motion (deltas), not `window_event`’s `CursorMoved`. Deltas are what we need for rotation; when the cursor is locked, many platforms don’t send meaningful position updates, but `DeviceEvent::MouseMotion { delta }` still delivers movement. Keyboard and mouse buttons are handled in `window_event`.

### Event lifecycle (one place)

| Method | When | What we do |
|--------|------|------------|
| **`resumed`** | Once when app is active | Create `AppDriver` (window + sizes), store in `self.driver`; on failure, `event_loop.exit()`. |
| **`device_event`** | Raw device input | If in game: `game_input.handle_mouse_motion(delta.0)` for look. |
| **`window_event`** | Every window event | See “Inside `window_event`” below; then handle `CloseRequested`, `RedrawRequested` (render), `Focused` (cursor lock). |
| **`about_to_wait`** | Before blocking | `driver.window().request_redraw()` to drive continuous redraws. |

**Inside `window_event`** (order matters):

1. Bail if `driver` is `None`.
2. **UI:** `manager.process_input(&event, driver.logical_cursor())` → list of `UIEvent`s.
3. **Game (if connected):** `game.poll()` → component updates and `ViewAction`s; apply updates; if `InGame`, `game_input.handle_game_input(&event)`; dispatch poll-produced `ViewAction`s.
4. **Views:** For each `UIEvent`, active view’s `handle_ui_events` → `ViewAction`s; dispatch via `handle_view_action(action, event_loop)`.
5. **Driver:** `driver.handle_winit_event(&event)` (cursor, resize).
6. **Special:** Match `event` — `CloseRequested` → `kill_game()`, `event_loop.exit()`; `RedrawRequested` → throttled game input send (when InGame), then `manager.update_components()` and `driver.render()`; `Focused` → lock/unlock cursor for game view.

So: **one pipeline** turns winit events into UI events and view actions, polls the client and applies updates, then handles close/redraw/focus.

### View actions

`handle_view_action(action, event_loop)` is the single place that reacts to `ViewAction`s (from views or from `game.poll()`):

| ViewAction | App behavior |
|------------|--------------|
| `UpdateComponent(updates)` | `manager.apply_updates(updates)` |
| `SwitchTo(view_id)` | `activate_view(&view_id)` |
| `QuitApp` | `kill_game()`, then `event_loop.exit()` |
| `SaveUsername` / `SaveMaze` / `SaveServer` | Persist to config |
| **`HostGame { game_name, maze, target_score }`** | `host_game(...)` (start server, then join as client) |
| **`JoinGame(server_addr)`** | `join_game(server_addr)` |
| `LeaveLobby` | `kill_game()`, then main menu |
| `SendChatMessage` | `game.send_chat_msg(...)`, clear chat input |
| **`SendStartGame`** | `send_start_game()` → `game.send_game_start()` (host only) |
| `StartGame` | `activate_view(views::GAME)` |
| `GameEnd(winner)` | Clear game input, lobby view, append winner to chat |

Host/join are triggered by the host/join menu views emitting `HostGame` / `JoinGame` from their form state; App only calls `host_game` / `join_game`.

### Threading

- **Server:** `start_server(...)` creates `GameServer`, spawns a thread running `server.run(cmd_rx)`. App keeps `ServerHandle` (send `Shutdown` to stop).
- **Client:** `Game::connect(...)` does join handshake, then `start_game_net(...)` spawns the client net thread. `Game` holds `GameNetHandle` (commands in, events out, join on drop/kill).

---

## 3. Game server and client (detail)

App owns **`server: Option<ServerHandle>`** (only when hosting) and **`game: Option<Game>`** (when connected as host or joiner). The server runs in its own thread; the client runs on the main thread with a separate network thread.

**Entry points:** Host/join/start/leave are all driven by `ViewAction` → `host_game`, `join_game`, `send_start_game`, `kill_game`. Per-frame behavior is already described above: `game.poll()`, `game_input.handle_game_input` / `handle_mouse_motion`, and on `RedrawRequested` (InGame) throttled `game_input.snapshot()` + `game.send_game_input(payload)`.

### Server: startup

- **`host_game(game_name, maze, target_score)`** validates (name, target score, maze connectivity, spawn points). If `app.server.is_none()`, it calls **`start_server(...)`**.
- **`host_game`** then calls **`join_game(public_addr)`** so the host’s App has both `server` and `game`.

**Server thread setup (`start_server` in `app.rs`):**

1. **Maze:** Clone the maze and call **`maze.set_spawn_points()`** (used for in-game spawns).
2. **GameServer:** **`GameServer::new(bind_addr, game_name, maze, target_score, host_username)`** — **`ServerSocket::bind(bind_addr, client_timeout)`** (UDP), parse `target_score`, build `GameServer { socket, clients: ClientList::new(), bullets, game_name, maze, target_score, host_username, state: Lobby }`. **`server.public_addr()`** gives the address string for the lobby.
3. **Channel:** **`let (cmd_tx, cmd_rx) = mpsc::channel::<ServerCommand>()`**. The server loop will only ever receive **`Shutdown`**; the main thread keeps **`cmd_tx`** to send it.
4. **Spawn:** **`thread::spawn(move || { catch_unwind(|| server.run(cmd_rx)) })`**. **`server`** and **`cmd_rx`** are moved into the new thread; the thread runs **`GameServer::run(cmd_rx)`** until it receives **`ServerCommand::Shutdown`**. Panics are caught and logged so the process doesn’t abort.
5. **Return:** **`Ok((ServerHandle { cmd_tx, join }, public_addr))`**. App stores **`ServerHandle`** in **`app.server`** and uses **`public_addr`** in the lobby; to stop the server, App (or `kill_game`) calls **`server.shutdown()`**, which sends **`Shutdown`** on **`cmd_tx`** and **`join.join()`**.

### Server: main loop

**`GameServer::run(cmd_rx)`** runs until **`ServerCommand::Shutdown`**:

1. **Shutdown:** `cmd_rx.try_recv()`; if `Shutdown`, break.
2. **Receive:** Drain `socket.recv()`; for each `(msg, src)` call **`handle_message`**:
   - **JoinGame** → `handle_join_msg`: add `Client`, send GameInfo, broadcast client list; if already InGame, spawn client.
   - **ChatMessage** → broadcast chat.
   - **StartGame** → see “When server goes into game” below.
   - **GameInput** → decode payload, set `client.last_input` and `client.last_seq` (drop old/duplicate seq).
   - **DisconnectNotice** → remove client, broadcast list.
   - **Ping** → send Pong.
3. **Resend** reliable packets; **remove stale clients**, then **remove_client** + broadcast list if any.
4. **If `state == GameState::InGame`:**  
   **`update_players(frame_time)`** (movement from `last_input`, shooting, invincibility) → **`update_bullets(frame_time)`** (advance, collision, scoring; if `target_score` reached → winner) → if winner **`handle_game_over`** (state = Lobby, broadcast GameEnd, reset clients), else **`broadcast_game_snapshot()`** (unreliable).
5. **Sleep** to maintain tick rate.

### When the server goes into game

- Host clicks “Start game” → App calls **`game.send_game_start()`** (reliable **StartGame**).
- Server in **`handle_start_game_msg`**: **`game_on_init(&mut self.clients, &self.maze)`** (spawn at random points); verify sender is host; send Ack and **broadcast reliable GameStart**; **`self.state = GameState::InGame`**.

From then on each tick: **update_players** → **update_bullets** → **broadcast_game_snapshot** or **handle_game_over**.

### Client: connection

- **`join_game(server_addr)`** → **`Game::connect(server_addr, "0.0.0.0:0", CONNECTION_TIMEOUT, username)`**:
  1. **`ClientSocket::new(...)`**, send **JoinGame** (username).
  2. Loop until **GameInfo** (or ConnectDeny/timeout): recv, decode_game_info, send Ack.
  3. **`start_game_net(socket, server_addr)`** → spawn net thread, get **`GameNetHandle`**.
  4. Build **`Game`** with game_name, maze, target_score, state from GameInfo; empty players/bullets; **interpolation state**: `prev_snapshot`, `current_snapshot`, `interpolation_alpha`, `snapshot_time`, `last_seq`.

App then sets **`app.game = Some(game)`**, activates lobby (or game view if state was already InGame), and sets up lobby UI.

**Client net: why two channels (cmd vs evt).**

The client network thread talks to the main thread over two **mpsc** channels:

- **`cmd_tx` → `cmd_rx` (GameNetCommand):** main thread → net thread. The main thread is the only producer: it sends **Send(msg)**, **SendReliable(msg)**, or **Shutdown**. The net thread drains **cmd_rx** with **try_recv()** each loop and performs UDP sends or exits. So the main thread never blocks on network I/O when sending; it only enqueues commands.

- **`evt_tx` → `evt_rx` (GameNetEvent):** net thread → main thread. The net thread is the only producer: when it receives UDP (ClientList, GameStart, Snapshot, Chat, Disconnected), it pushes a **GameNetEvent** on **evt_tx**. The main thread drains **evt_rx** with **try_recv()** inside **`game.poll()`** each frame. So the main thread never blocks on **recv**; the net thread does the blocking socket read and forwards results as events.

Two channels keep the two directions separate: different message types (commands vs events), clear ownership (main holds **cmd_tx** and **evt_rx**; net thread holds **cmd_rx** and **evt_tx**), and non-blocking APIs on the main thread (**send** on cmd, **try_recv** on evt). A single shared channel would require both sides to send and receive the same enum and would not remove the need for two connections; splitting by direction matches “main tells net what to send” and “net tells main what arrived.”

### Client: snapshot handling (important logic)

- **Network thread** receives **GameSnapShot** → sends **`GameNetEvent::Snapshot(msg)`** on `evt_tx`.
- **Main thread** in **`game.poll()`**: drain `net_handle.try_recv()`. For **`GameNetEvent::Snapshot(msg)`** when **state == InGame**:
  - Decode **GameSnapShotPayload**; if seq is newer than **last_seq**: **last_seq** = seq, **prev_snapshot = current_snapshot**, **current_snapshot = payload**, **snapshot_time = Instant::now()**.
- **`update_interpolation()`** (each poll in InGame):
  - **alpha = elapsed_since(snapshot_time) / SNAPSHOT_INTERVAL** (capped).
  - **interpolated = interpolate_snapshots(prev, current, alpha)** (positions and angles lerped).
  - **apply_snapshot(interpolated)** → update local **players** (pos, score, status) and **bullets**.
- **poll()** then builds **ComponentUpdate**s (leaderboard, minimap, game render/camera) from current **players** and **bullets** and returns them to App for **apply_updates**.

So snapshots are **sequence-checked**, **prev/current** are updated only on newer seq, and the **interpolated** state is what the UI renders, giving smooth motion between server ticks.

### Game input: capture and send (important logic)

**Capture:**

- **Keys and mouse button** (WASD, Shift, Shoot): in **`window_event`**, when **InGame**, **`game_input.handle_game_input(&event)`** updates **actions** (insert/remove `PlayerAction`), **mouse_left_down**, **left_shift_down**.
- **Mouse delta (look):** in **`device_event`**, when in game, **`game_input.handle_mouse_motion(delta.0)`** adds to **mouse_dx**.

So **game_input** is the single accumulator for actions, shift, and mouse_dx.

**Snapshot and send:**

- On **`RedrawRequested`**, when **game** is Some and **state == InGame**, throttle: **`last_input_send.elapsed() >= INPUT_SEND_INTERVAL`** (or `last_input_send` is None).
- When it’s time: **`payload = game_input.snapshot()`** (builds **GameInputPayload** from current state and **resets mouse_dx to 0**); **`game.send_game_input(payload)`** (builds Message with seq, pushes **GameNetCommand::Send(msg)** to net thread); **last_input_send = Some(Instant::now())**.
- Net thread sends UDP **GameInput**. So input is sent **at most once per INPUT_SEND_INTERVAL** (aligned with server tick); each send carries the **current** action set and **accumulated mouse_dx** since the previous send; **mouse_dx** is cleared after each snapshot.

**Server side:** **handle_game_input** stores **client.last_input** and **client.last_seq**. **update_players(dt)** uses **last_input** (and maze, bullets) in **update_player** for movement and shooting; the next **broadcast_game_snapshot** reflects that input.

---

## 4. Reference

**Dependencies:** fps_ui (AppDriver, UIManager, layout/rendering), fps_net (sockets, Message, protocol), fps_levels (Maze), fps_config (Config), winit (ApplicationHandler, window/cursor).

**Constants and IDs:** Use `super::constants::*` (e.g. `server::DEFAULT_BIND_ADDR`, `client::CONNECTION_TIMEOUT`, `tick_rate::TICK_RATE_HZ`) and **`view_ids::views`** / **`view_ids::components`** instead of string literals.
