//! # Game Client Module
//!
//! Manages client-side game state and networking integration. Handles connection
//! to the server, game state synchronization, and UI updates based on server messages.
//!
//! The client maintains a local copy of the game state (players, bullets, maze)
//! that is updated from server snapshots. It also handles sending player inputs
//! to the server.

use std::net::SocketAddr;
use std::time::{Duration, Instant};
use std::collections::{HashMap, HashSet};

use crate::app::{
    view_ids::components,
    ClientStatus,
    game_structs::Bullet,
};
use crate::view::ViewAction;
use fps_levels::maze::Maze;
use fps_net::message::GameInputPayload;
use fps_net::{
    ClientSocket, Message, MessageType, ignore_would_block,
    message::{ChatMessagePayload, GameInfoPayload, JoinGamePayload, GameSnapShotPayload, PlayerSnapshot},
};
use fps_ui::ComponentUpdate;

use super::game_client_net::{GameNetEvent, GameNetHandle, start_game_net};
use super::{Client, GameState, Pos};

/// Represents the client's view of the game state.
///
/// Maintains local copies of players, bullets, and game configuration received
/// from the server. Handles network communication via a separate networking thread.
#[derive(Debug)]
pub struct Game {
    pub _server_addr: SocketAddr,
    pub player_name: String,
    pub last_seq: u32,

    // --- game settings from server ---
    pub game_name: String,
    pub maze: Maze,
    pub target_score: u32,
    pub players: HashMap<String, Client>,
    pub bullets: Vec<Bullet>,
    pub state: GameState,
    pub is_host: bool,

    // --- networking ---
    pub net_handle: GameNetHandle,
    
    // --- interpolation state ---
    /// Previous snapshot state for interpolation (positions before current)
    prev_snapshot: GameSnapShotPayload,
    /// Current snapshot state (latest received)
    current_snapshot: GameSnapShotPayload,
    /// Interpolation progress (0.0 = prev, 1.0 = current)
    interpolation_alpha: f32,
    /// Time when current snapshot was received
    snapshot_time: Instant,
}

impl Game {
    /// Connects to a game server and initializes the game state.
    ///
    /// Performs the connection handshake:
    /// 1. Creates a client socket
    /// 2. Sends a join game request
    /// 3. Waits for game info response
    /// 4. Starts the networking thread
    ///
    /// # Arguments
    ///
    /// * `server_addr` - Address of the server to connect to
    /// * `local_addr` - Local address to bind the socket to
    /// * `timeout` - Connection timeout duration
    /// * `username` - Username to use for joining
    ///
    /// # Returns
    ///
    /// A new `Game` instance if connection succeeds, error otherwise.
    pub fn connect(
        server_addr: SocketAddr,
        local_addr: &str,
        timeout: Duration,
        username: String,
    ) -> Result<Self, Box<dyn std::error::Error>> {
        // Use provided timeout or default
        let cur_username = username.clone();

        // 1. Create client socket
        let mut socket = ClientSocket::new(local_addr, &server_addr.to_string(), timeout)?;

        // 2. Send JoinGame
        let join_payload = JoinGamePayload { username: username.clone() };
        let join_msg = Message::new_join_game(socket.next_sequence(), &join_payload);
        socket.send(&join_msg)?;

        // 3. Wait for GameInfo (handshake)
        let start = Instant::now();
        let game_info: GameInfoPayload;

        loop {
            // Timeout protection
            if start.elapsed() > timeout {
                return Err("Timed out waiting for GameInfo from server".into());
            }

            match socket.recv() {
                Ok(Some(msg)) if msg.header.msg_type == MessageType::GameInfo => {
                    game_info = msg.decode_game_info()?;
                    let ack_msg = Message::new_ack(socket.next_sequence(), msg.header.sequence);
                    socket.send(&ack_msg)?;
                    break;
                }
                Ok(Some(
                    msg)) if msg.header.msg_type == MessageType::ConnectDeny => {
                    let reason = msg.decode_connect_deny()?;
                    eprintln!("Server denied connection: {}", reason);
                    return Err(format!("Connection denied by server: {}", reason).into());
                }

                Ok(Some(_)) => {
                    // Ignore other messages during handshake
                }

                Ok(None) => {
                    // No message yet
                }
                
                Err(e) => {
                    if let Err(e) = ignore_would_block(e) {
                        return Err(e);
                    }
                }
            }

            std::thread::sleep(Duration::from_millis(5));
        }

        let net_handle: GameNetHandle = start_game_net(socket, server_addr);
        let state: GameState = game_info
            .state
            .parse()
            .map_err(|_| format!("Invalid game state from server"))?;

        // 4. Construct fully-initialized Game
        Ok(Self {
            _server_addr: server_addr,
            player_name: username,
            game_name: game_info.game_name,
            last_seq: 0,
            maze: game_info.maze,
            target_score: game_info.target_score,
            players: HashMap::new(),
            bullets: Vec::new(),
            state,
            is_host: cur_username == game_info.host_username,
            net_handle,
            prev_snapshot: GameSnapShotPayload::default(),
            current_snapshot: GameSnapShotPayload::default(),
            interpolation_alpha: 1.0,
            snapshot_time: Instant::now(),
        })
    }

    /// Sends a chat message to the server via the networking thread.
    ///
    /// # Arguments
    ///
    /// * `text` - The message text to send
    /// * `username` - The username to attach to the message
    ///
    /// # Returns
    ///
    /// `Ok(())` if sent successfully, error if channel is closed.
    pub fn send_chat_msg(
        &mut self,
        text: &str,
        username: &str,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let payload = ChatMessagePayload {
            username: username.to_string(),
            text: text.to_string(),
        };

        let seq = self.net_handle.next_sequence();
        let msg = Message::new_chat_message(seq, &payload);
        self.net_handle.send(msg)?;
        Ok(())
    }

    /// Sends a game start request to the server (host only).
    ///
    /// Only the host can start the game. This sends a reliable message
    /// requesting the server to transition from lobby to game state.
    ///
    /// # Returns
    ///
    /// `Ok(())` if sent (or if not host), error if channel is closed.
    pub fn send_game_start(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        if self.is_host {
            let seq = self.net_handle.next_sequence();
            let msg = Message::new_game_start(seq);
            self.net_handle.send_reliable(msg)?;
        }
        Ok(())
    }

    pub fn send_game_input(&mut self, payload: GameInputPayload) -> Result<(), Box<dyn std::error::Error>> {
        let seq = self.net_handle.next_sequence();
        let msg = Message::new_game_input(seq, &payload);
        self.net_handle.send(msg)?;
        Ok(())
    }

    /// Polls for network events and updates game state.
    ///
    /// Processes incoming network messages, updates local game state (players,
    /// bullets), and generates UI updates. Should be called each frame.
    ///
    /// # Returns
    ///
    /// A tuple of:
    /// - `Vec<ComponentUpdate>`: UI component updates to apply
    /// - `Vec<ViewAction>`: View actions (e.g., view changes)
    pub fn poll(&mut self) -> (Vec<ComponentUpdate>, Vec<ViewAction>) {
        let mut ui_updates: Vec<ComponentUpdate> = Vec::new();
        let mut view_actions: Vec<ViewAction> = Vec::new();

        // Drain network events directly
        while let Some(evt) = self.net_handle.try_recv() {
            match (&self.state, evt) {
                (GameState::Lobby, GameNetEvent::Chat(msg)) => {
                    if let Ok(chat) = msg.decode_chat_message() {
                        ui_updates.push(ComponentUpdate::AppendText(
                            components::LOBBY_CHAT_LOG.into(),
                            format!("{}: {}\n", chat.username, chat.text),
                        ));
                    }
                }

                (GameState::Lobby, GameNetEvent::ClientList(users)) => {
                    ui_updates.extend(self.update_player_list(users));
                }

                (GameState::Lobby, GameNetEvent::GameStart) => {
                    self.state = GameState::InGame;
                    ui_updates.push(ComponentUpdate::AppendText(
                        components::LOBBY_CHAT_LOG.into(),
                        format!("Game starting"),
                    ));
                    view_actions.push(ViewAction::StartGame);
                }

                (GameState::InGame, GameNetEvent::Snapshot(msg)) => {
                    self.state = GameState::InGame;
                    // Handle sequence wrapping: if difference is less than half of u32::MAX,
                    // treat it as a newer packet (handles wrap-around correctly)
                    let seq_diff = msg.header.sequence.wrapping_sub(self.last_seq);
                    if seq_diff > 0 && seq_diff < 0x8000_0000 {
                        self.last_seq = msg.header.sequence;
                        if let Ok(payload) = msg.decode_game_snapshot() {
                            // Move current to previous, new snapshot becomes current
                            self.prev_snapshot = std::mem::take(&mut self.current_snapshot);
                            self.current_snapshot = payload;
                            self.snapshot_time = Instant::now();
                            // Note: interpolation_alpha will be recalculated from time in update_interpolation()
                        }
                    }
                }


                (GameState::InGame, GameNetEvent::GameEnd(winner)) => {
                    self.state = GameState::Lobby;
                    view_actions.push(ViewAction::GameEnd(winner));
                }

                (_, GameNetEvent::Disconnected) => {
                    self.state = GameState::Disconnected;
                    view_actions.push(ViewAction::LeaveLobby);
                }

                _ => {}
            }
        }

        // Update interpolation and apply interpolated state
        if self.state == GameState::InGame {
            self.update_interpolation();
            ui_updates.extend(self.update_leaderboard());
            ui_updates.extend(self.update_mini_map());
            ui_updates.extend(self.update_game_render());
        }
        
        (ui_updates, view_actions)
    }

    /// Update the lobby player list and generate UI updates
    pub fn update_player_list(&mut self, new_usernames: Vec<String>) -> Vec<ComponentUpdate> {
        let mut updates = Vec::new();

        // Convert current players to a set of usernames for easy diff
        let old_usernames: HashSet<String> = self.players.keys().cloned().collect();
        let new_usernames_set: HashSet<String> = new_usernames.iter().cloned().collect();

        // --- Players who joined ---
        for username in new_usernames_set.difference(&old_usernames) {
            self.players.insert(username.clone(), Client::new(username.clone()));

            // Add system chat message
            let msg = format!("{} joined the lobby.", username);
            updates.push(ComponentUpdate::AppendText(components::LOBBY_CHAT_LOG.into(), msg));
        }

        // --- Players who left ---
        for username in old_usernames.difference(&new_usernames_set) {
            self.players.remove(username);

            // Add system chat message
            let msg = format!("{} left the lobby.", username);
            updates.push(ComponentUpdate::AppendText(components::LOBBY_CHAT_LOG.into(), msg));
        }

        // --- Update the user list UI ---
        let usernames_vec: Vec<String> = self.players.keys().cloned().collect();
        updates.push(ComponentUpdate::SetTextVec(
            components::LOBBY_USER_LIST.into(),
            usernames_vec,
        ));

        updates
    }

    fn apply_snapshot(&mut self, snapshot: GameSnapShotPayload) {
        self.players.retain(|id, _| snapshot.players.contains_key(id));

        for (id, snap) in snapshot.players {
            let client = self.players.entry(id.clone())
                .or_insert_with(|| Client::new(id));

            client.pos.x = snap.pos.0;
            client.pos.y = snap.pos.1;
            client.pos.angle = snap.pos.2;
            client.score = snap.score;
            client.status = if snap.is_invincible {ClientStatus::Invincible(1.0)} else {ClientStatus::Normal};
        }

        self.bullets = snapshot.bullets.iter().map(|b| {
            Bullet{
                owner_id: String::new(),
                pos: Pos{
                    x: b.pos.0, y: b.pos.1, angle: b.pos.2}
                }
        }).collect();
    }

    fn update_mini_map(&self) -> Vec<ComponentUpdate> {
        let local_client = self.players.get(&self.player_name).unwrap();
        vec![
            ComponentUpdate::SetMiniMapPlayer(
                components::GAME_MINI_MAP.to_string(),
                local_client.pos.to_tuple(),
            )
        ]
    }

    fn update_leaderboard(&self) -> Vec<ComponentUpdate> {
        let mut updates = Vec::new();

        // --- Update top 3 leaderboard ---
        let mut entries: Vec<_> = self.players.values().collect();
        entries.sort_by(|a, b| b.score.cmp(&a.score));
        let top_entries = entries.iter().take(3);

        let leaderboard_lines: Vec<String> = top_entries
            .map(|c| format!("{} : {}", c.id, c.score))
            .collect();

        updates.push(ComponentUpdate::SetTextVec(
            components::GAME_LEADERBOARD.into(),
            leaderboard_lines,
        ));

        // --- Update current player's score label ---
        if let Some(local_client) = self.players.get(&self.player_name) {
            updates.push(ComponentUpdate::SetText(
                components::GAME_SCORE_LABEL.into(),
                format!("{} : {}", self.player_name, local_client.score),
            ));
        }

        updates
    }

    fn update_game_render(&self) -> Vec<ComponentUpdate> {
        let mut updates = Vec::new();

        // --- players ---
        let mut players_map = HashMap::new();
        for (id, client) in &self.players {
            // Use matches! for the invincibility check
            let is_invincible = matches!(client.status, ClientStatus::Invincible(_));
            
            players_map.insert(
                id.clone(),
                (
                    client.pos.x, 
                    client.pos.y, 
                    client.pos.angle,
                    is_invincible,
                ),
            );
        }

        // --- bullets ---
        // Convert your bullets list into a simple tuple format for the UI
        let bullets_list: Vec<(f32, f32, f32)> = self.bullets
            .iter()
            .map(|b| (b.pos.x, b.pos.y, b.pos.angle))
            .collect();

        // Push combined update (assuming SetGameRender can take bullets now)
        updates.push(ComponentUpdate::SetGameRender(
            components::GAME_RENDER.to_string(),
            players_map,
            bullets_list, // Add bullets to the message
        ));

        // --- camera ---
        if let Some(client) = self.players.get(&self.player_name) {
            updates.push(ComponentUpdate::SetGameRenderCamera(
                components::GAME_RENDER.to_string(),
                (
                    client.id.clone(),
                    client.pos.x,
                    client.pos.y,
                    client.pos.angle,
                    matches!(client.status, ClientStatus::Invincible(_))
                ),
            ));
        }

        updates
    }

    /// Updates interpolation and applies interpolated snapshot using time-based calculation.
    ///
    /// Uses elapsed time since the current snapshot was received to calculate interpolation
    /// alpha, making it independent of frame rate. Includes a jitter buffer to smooth out
    /// network packet arrival timing, especially important for high-latency connections.
    fn update_interpolation(&mut self) {
        use super::constants::client;
        
        // If we have both previous and current snapshots (prev not empty), interpolate
        if !self.prev_snapshot.players.is_empty() {
            // Calculate alpha based on actual elapsed time, not frame count
            let elapsed = self.snapshot_time.elapsed().as_secs_f32();
            let jitter_buffer = client::INTERPOLATION_JITTER_BUFFER_SECONDS;
            let snapshot_interval = client::SNAPSHOT_INTERVAL_SECONDS;
            
            // Calculate interpolation alpha: 0.0 at start of buffer, 1.0 after full interval
            // The jitter buffer delays interpolation start to absorb late packets
            let alpha = if elapsed < jitter_buffer {
                // Wait for jitter buffer before starting interpolation
                0.0
            } else {
                // Interpolate from 0.0 to 1.0 over the snapshot interval
                ((elapsed - jitter_buffer) / snapshot_interval).clamp(0.0, 1.0)
            };
            
            // Store calculated alpha for potential future use
            self.interpolation_alpha = alpha;
            
            // Create interpolated snapshot
            let interpolated = self.interpolate_snapshots(&self.prev_snapshot, &self.current_snapshot, alpha);
            self.apply_snapshot(interpolated);
        } else {
            // Only current snapshot available (prev is empty), apply directly
            self.apply_snapshot(self.current_snapshot.clone());
        }
    }
    
    /// Interpolates between two snapshots
    fn interpolate_snapshots(
        &self,
        prev: &GameSnapShotPayload,
        current: &GameSnapShotPayload,
        alpha: f32,
    ) -> GameSnapShotPayload {
        let mut interpolated_players = HashMap::new();
        
        // Interpolate player positions
        for (id, current_snap) in &current.players {
            if let Some(prev_snap) = prev.players.get(id) {
                // Interpolate position
                let x = prev_snap.pos.0 + (current_snap.pos.0 - prev_snap.pos.0) * alpha;
                let y = prev_snap.pos.1 + (current_snap.pos.1 - prev_snap.pos.1) * alpha;
                
                // Interpolate angle (handle wrapping)
                let mut angle = prev_snap.pos.2;
                let angle_diff = current_snap.pos.2 - prev_snap.pos.2;
                // Normalize angle difference to [-PI, PI] range
                let angle_diff = if angle_diff > std::f32::consts::PI {
                    angle_diff - 2.0 * std::f32::consts::PI
                } else if angle_diff < -std::f32::consts::PI {
                    angle_diff + 2.0 * std::f32::consts::PI
                } else {
                    angle_diff
                };
                angle += angle_diff * alpha;
                
                interpolated_players.insert(id.clone(), PlayerSnapshot {
                    pos: (x, y, angle),
                    score: current_snap.score, // Don't interpolate score
                    is_invincible: current_snap.is_invincible, // Use current state
                });
            } else {
                // New player, use current snapshot
                interpolated_players.insert(id.clone(), current_snap.clone());
            }
        }
        
        // For bullets, use current snapshot (they move too fast to interpolate meaningfully)
        GameSnapShotPayload {
            players: interpolated_players,
            bullets: current.bullets.clone(),
        }
    }

    pub fn kill(self) {
        self.net_handle.shutdown();
    }
}
