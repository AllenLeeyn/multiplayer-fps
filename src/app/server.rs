use std::error::Error;
use std::net::SocketAddr;
use std::sync::mpsc;
use std::thread;
use std::time::{Duration, SystemTime};

use super::{Client, ClientList};
use fps_levels::maze::Maze;
use fps_net::ignore_would_block;
use fps_net::message::{ChatMessagePayload, ClientListPayload, GameInfoPayload, Message};
use fps_net::protocol::MessageType;
use fps_net::server_socket::ServerSocket;

/// Commands sent *to* the server thread
pub enum ServerCommand {
    Shutdown,
}

/// Handle owned by the App
pub struct ServerHandle {
    pub cmd_tx: mpsc::Sender<ServerCommand>,
    pub join: thread::JoinHandle<()>,
}

impl ServerHandle {
    pub fn shutdown(self) {
        let _ = self.cmd_tx.send(ServerCommand::Shutdown);
        let _ = self.join.join();
    }
}

/// Minimal server struct
pub struct GameServer {
    socket: ServerSocket,
    clients: ClientList,

    // --- Game configuration ---
    pub game_name: String,
    pub maze: Maze,
    pub target_score: String,
}

impl GameServer {
    pub fn new(
        bind_addr: &str,
        game_name: String,
        maze: Maze,
        target_score: String,
    ) -> Result<Self, Box<dyn std::error::Error>> {
        let socket = ServerSocket::bind(bind_addr, Duration::from_secs(5))?;

        Ok(Self {
            socket,
            clients: ClientList::new(),
            game_name,
            maze,
            target_score,
        })
    }

    /// Remove a client when they disconnect or timeout
    pub fn remove_client(&mut self, addr: &SocketAddr) {
        self.clients.remove_client(addr);
    }

    pub fn public_addr(&self) -> Result<String, Box<dyn Error>> {
        self.socket.public_addr()
    }

    /// Broadcast a message to all connected clients
    pub fn broadcast(&mut self, msg: &Message) {
        if let Err(failed_addrs) = self.socket.broadcast(msg) {
            eprintln!(
                "Failed to broadcast message to some clients: {:?}",
                failed_addrs
            );
        }
    }

    /// Broadcast a chat message to all connected clients
    pub fn broadcast_chat(&mut self, username: &str, text: &str) {
        let payload = ChatMessagePayload {
            username: username.to_string(),
            text: text.to_string(),
        };

        let msg = Message::new_chat_message(self.socket.next_sequence(), &payload);

        self.broadcast(&msg);
    }

    fn broadcast_client_list(&mut self) {
        let clients = self
            .clients
            .all_clients()
            .iter()
            .map(|c| c.id.clone())
            .collect();

        let payload = ClientListPayload { clients };

        let msg = Message::new_client_list(self.socket.next_sequence(), &payload);
        self.broadcast(&msg);
    }

    pub fn run(mut self, cmd_rx: mpsc::Receiver<ServerCommand>) {
        let tick_duration = Duration::from_millis(33); // ~1/30th of a second
        let started_at = SystemTime::now();
        println!(
            "[server][{:?}] Server started at {:?}",
            started_at,
            self.socket.public_addr()
        );

        loop {
            let loop_start = std::time::Instant::now();

            // Check for shutdown command
            if let Ok(ServerCommand::Shutdown) = cmd_rx.try_recv() {
                println!("[server] Shutdown requested");
                break;
            }

            // Receive messages from clients
            match self.socket.recv() {
                Ok(Some((msg, src))) => {
                    self.handle_message(msg, src);
                }
                Ok(None) => { /* No messages */ }
                Err(e) => {
                    if let Err(e) = ignore_would_block(e) {
                        eprintln!("Error receiving message: {}", e);
                    }
                }
            }

            // Resend reliable packets
            self.socket.resend_pending();

            // Remove stale clients
            let removed = self.socket.remove_stale_clients();
            for addr in removed.clone() {
                self.remove_client(&addr);
                println!("Removed stale client: {}", addr);
            }
            if !removed.is_empty() {
                self.broadcast_client_list();
            }

            // Sleep to maintain 30 Hz tick
            let elapsed = loop_start.elapsed();
            if elapsed < tick_duration {
                thread::sleep(tick_duration - elapsed);
            }
        }

        println!("[server] Exiting server thread");
    }

    /// Process a single received message
    fn handle_message(&mut self, msg: Message, src: SocketAddr) {
        match msg.header.msg_type {
            MessageType::JoinGame => self.handle_join_msg(msg, src),
            MessageType::ChatMessage => self.handle_chat_msg(msg, src),
            MessageType::DisconnectNotice => {
                self.remove_client(&src);
                self.broadcast_client_list();
            }
            MessageType::Ping => self.send_pong(src, msg.header.sequence),
            _ => {
                println!(
                    "Unhandled message type from {}: {:?}",
                    src, msg.header.msg_type
                );
            }
        }
    }

    /// Handle a JoinGame message
    fn handle_join_msg(&mut self, msg: Message, src: SocketAddr) {
        match msg.decode_join_game() {
            Ok(payload) => {
                if self.clients.len() >= self.maze.config.max_players() {
                    let msg = Message::new_connect_deny(self.socket.next_sequence(), "Lobby full");
                    let _ = self.socket.send(src, &msg);
                    return;
                }

                let client = Client::new(payload.username.clone());
                if !self.clients.add_client(client, src) {
                    let msg =
                        Message::new_connect_deny(self.socket.next_sequence(), "Duplicate name");
                    let _ = self.socket.send(src, &msg);
                    return;
                }

                println!("Player '{}' joined from {}", payload.username, src);

                // Send the game info back
                self.send_game_info(src);
                self.broadcast_client_list();
            }
            Err(e) => {
                eprintln!("Failed to decode JoinGamePayload from {}: {}", src, e);
            }
        }
    }

    /// Handle a ChatMessage message
    fn handle_chat_msg(&mut self, msg: Message, src: SocketAddr) {
        // Only process chat from registered clients
        if let Some(client) = self.clients.get(&src) {
            match msg.decode_chat_message() {
                Ok(payload) => {
                    println!("Chat from {}: {}", client.id, payload.text);

                    // Broadcast to all clients including sender
                    self.broadcast_chat(&client.id.clone(), &payload.text);
                }
                Err(e) => {
                    eprintln!("Failed to decode chat message from {}: {}", src, e);
                }
            }
        } else {
            eprintln!("Received chat message from unregistered client: {}", src);
        }
    }

    fn send_game_info(&mut self, addr: SocketAddr) {
        let game_info = GameInfoPayload {
            game_name: self.game_name.clone(),
            maze: self.maze.clone(),
            target_score: self.target_score.clone(),
        };

        let response = Message::new_game_info(self.socket.next_sequence(), &game_info);

        if let Err(e) = self.socket.send_reliable(addr, &response) {
            eprintln!("Failed to send GameInfo to {}: {}", addr, e);
        }
    }

    fn send_pong(&mut self, src: SocketAddr, seq: u32) {
        match self.socket.handle_ping(src, seq) {
            Ok((addr, pong_msg)) => {
                if let Err(e) = self.socket.send(addr, &pong_msg) {
                    eprintln!(
                        "[server] Failed to send pong to {} (seq={}): {}",
                        addr, seq, e
                    );
                }
            }
            Err(e) => {
                eprintln!(
                    "[server] Failed to handle ping from {} (seq={}): {}",
                    src, seq, e
                );
            }
        }
    }
}
