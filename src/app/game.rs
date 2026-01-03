use std::net::SocketAddr;
use std::time::{Duration, Instant};

use crate::view::ViewAction;
use fps_levels::maze::Maze;
use fps_net::{
    ClientSocket, Message, MessageType, ignore_would_block,
    message::{ChatMessagePayload, GameInfoPayload, JoinGamePayload},
};
use fps_ui::ComponentUpdate;

use super::client::Client;
use super::game_net::{GameNetEvent, GameNetHandle, start_game_net};

#[derive(Debug)]
pub struct Game {
    pub server_addr: SocketAddr,

    // --- game settings from server ---
    pub game_name: String,
    pub maze: Maze,
    pub target_score: String,
    pub players: Vec<Client>,

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
        // 1. Create client socket
        let mut socket = ClientSocket::new(local_addr, &server_addr.to_string(), timeout)?;

        // 2. Send JoinGame
        let join_payload = JoinGamePayload { username };
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
                Ok(Some(msg)) if msg.header.msg_type == MessageType::ConnectDeny => {
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

        // 4. Construct fully-initialized Game
        Ok(Self {
            server_addr,
            game_name: game_info.game_name,
            maze: game_info.maze,
            target_score: game_info.target_score,
            players: Vec::new(),
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

    pub fn poll(&mut self) -> (Vec<ComponentUpdate>, Vec<ViewAction>) {
        let mut ui_updates: Vec<ComponentUpdate> = Vec::new();
        let mut view_actions: Vec<ViewAction> = Vec::new();

        // Drain network events directly
        while let Some(evt) = self.net_handle.try_recv() {
            match evt {
                GameNetEvent::Chat(msg) => {
                    if let Ok(chat) = msg.decode_chat_message() {
                        ui_updates.push(ComponentUpdate::AppendText(
                            "lobby_chat_log".into(),
                            format!("{}: {}\n", chat.username, chat.text),
                        ));
                    }
                }

                GameNetEvent::ClientList(users) => {
                    ui_updates.extend(self.update_players(users));
                }

                GameNetEvent::Disconnected => {
                    view_actions.push(ViewAction::LeaveLobby);
                }

                _ => {}
            }
        }

        (ui_updates, view_actions)
    }

    /// Update the lobby player list and generate UI updates
    pub fn update_players(&mut self, new_usernames: Vec<String>) -> Vec<ComponentUpdate> {
        let mut updates = Vec::new();

        // Convert current players to a set of usernames for easy diff
        let old_usernames: std::collections::HashSet<_> =
            self.players.iter().map(|c| c.id.clone()).collect();
        let new_usernames_set: std::collections::HashSet<_> =
            new_usernames.iter().cloned().collect();

        // --- Players who joined ---
        for username in new_usernames_set.difference(&old_usernames) {
            // Add new Client to players
            self.players.push(Client::new(username.clone()));

            // Add system chat message
            let msg = format!("{} joined the lobby.", username);
            updates.push(ComponentUpdate::AppendText("lobby_chat_log".into(), msg));
        }

        // --- Players who left ---
        for username in old_usernames.difference(&new_usernames_set) {
            // Remove the client
            self.players.retain(|c| &c.id != username);

            // Add system chat message
            let msg = format!("{} left the lobby.", username);
            updates.push(ComponentUpdate::AppendText("lobby_chat_log".into(), msg));
        }

        // --- Update the user list UI ---
        let usernames_vec: Vec<String> = self.players.iter().map(|c| c.id.clone()).collect();
        updates.push(ComponentUpdate::SetTextVec(
            "lobby_user_list".into(),
            usernames_vec,
        ));

        updates
    }

    pub fn kill(self) {
        self.net_handle.shutdown();
    }
}
