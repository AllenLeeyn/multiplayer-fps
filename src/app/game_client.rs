use std::net::SocketAddr;
use std::time::{Duration, Instant};
use std::collections::{HashMap, HashSet};

use crate::app::ClientStatus;
use crate::app::game_structs::Bullet;
use crate::view::ViewAction;
use fps_levels::maze::Maze;
use fps_net::message::GameInputPayload;
use fps_net::{
    ClientSocket, Message, MessageType, ignore_would_block,
    message::{ChatMessagePayload, GameInfoPayload, JoinGamePayload, GameSnapShotPayload},
};
use fps_ui::ComponentUpdate;

use super::game_client_net::{GameNetEvent, GameNetHandle, start_game_net};
use super::{Client, GameState, Pos};

#[derive(Debug)]
pub struct Game {
    pub _server_addr: SocketAddr,
    pub player_name: String,

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
}

impl Game {
    pub fn connect(
        server_addr: SocketAddr,
        local_addr: &str,
        timeout: Duration,
        username: String,
    ) -> Result<Self, Box<dyn std::error::Error>> {
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
            maze: game_info.maze,
            target_score: game_info.target_score,
            players: HashMap::new(),
            bullets: Vec::new(),
            state,
            is_host: cur_username == game_info.host_username,
            net_handle,
        })
    }

    /// Send a chat message via the networking thread
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

    pub fn poll(&mut self) -> (Vec<ComponentUpdate>, Vec<ViewAction>) {
        let mut ui_updates: Vec<ComponentUpdate> = Vec::new();
        let mut view_actions: Vec<ViewAction> = Vec::new();

        // Drain network events directly
        while let Some(evt) = self.net_handle.try_recv() {
            match (&self.state, evt) {
                (GameState::Lobby, GameNetEvent::Chat(msg)) => {
                    if let Ok(chat) = msg.decode_chat_message() {
                        ui_updates.push(ComponentUpdate::AppendText(
                            "lobby_chat_log".into(),
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
                        "lobby_chat_log".into(),
                        format!("Game starting"),
                    ));
                    view_actions.push(ViewAction::StartGame);
                }

                (GameState::InGame, GameNetEvent::Snapshot(msg)) => {
                    self.state = GameState::InGame;
                    if let Ok(payload) = msg.decode_game_snapshot() {
                        self.apply_snapshot(payload);
                        ui_updates.extend(self.update_leaderboard());
                        ui_updates.extend(self.update_mini_map());
                        ui_updates.extend(self.update_game_render());
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
            updates.push(ComponentUpdate::AppendText("lobby_chat_log".into(), msg));
        }

        // --- Players who left ---
        for username in old_usernames.difference(&new_usernames_set) {
            self.players.remove(username);

            // Add system chat message
            let msg = format!("{} left the lobby.", username);
            updates.push(ComponentUpdate::AppendText("lobby_chat_log".into(), msg));
        }

        // --- Update the user list UI ---
        let usernames_vec: Vec<String> = self.players.keys().cloned().collect();
        updates.push(ComponentUpdate::SetTextVec(
            "lobby_user_list".into(),
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
                "game_mini_map".to_string(),
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
            "game_leaderboard".into(),
            leaderboard_lines,
        ));

        // --- Update current player's score label ---
        if let Some(local_client) = self.players.get(&self.player_name) {
            updates.push(ComponentUpdate::SetText(
                "game_score_label".into(),
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
            "game_render".to_string(),
            players_map,
            bullets_list, // Add bullets to the message
        ));

        // --- camera ---
        if let Some(client) = self.players.get(&self.player_name) {
            updates.push(ComponentUpdate::SetGameRenderCamera(
                "game_render".to_string(),
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

    pub fn kill(self) {
        self.net_handle.shutdown();
    }
}
