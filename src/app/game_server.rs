use std::error::Error;
use std::net::SocketAddr;
use std::sync::mpsc;
use std::thread;
use std::time::{Duration, SystemTime, Instant};

use super::{Client, ClientList, GameState, 
    game::{
        game_on_init,
        spawn_client_randomly,
        update_player,
        update_bullet,
        handle_bullet_hit,
    },
    game_structs::{Bullet, BulletStatus}
};

use fps_levels::maze::Maze;
use fps_net::ignore_would_block;
use fps_net::message::{
    ChatMessagePayload,
    ClientListPayload, 
    GameInfoPayload,
    Message,
    GameSnapShotPayload,
    BulletSnapshot,
};
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

pub struct GameServer {
    socket: ServerSocket,
    clients: ClientList,
    bullets: Vec<Bullet>,

    // --- Game configuration ---
    pub game_name: String,
    pub maze: Maze,
    pub target_score: u32,
    pub host_username: String,
    pub state: GameState,
}

impl GameServer {
    pub fn new(
        bind_addr: &str,
        game_name: String,
        maze: Maze,
        target_score: String,
        host_username: String,
    ) -> Result<Self, Box<dyn std::error::Error>> {
        let socket = ServerSocket::bind(bind_addr, Duration::from_secs(5))?;

        let target_score = target_score
            .parse::<u32>()
            .map_err(|_| "Target score must be a number".to_string())?;

        Ok(Self {
            socket,
            clients: ClientList::new(),
            bullets: Vec::new(),
            game_name,
            maze,
            target_score,
            host_username,
            state: GameState::Lobby,
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

    pub fn broadcast_reliable(&mut self, msg: &Message) {
        if let Err(failed_addrs) = self.socket.broadcast_relibale(msg) {
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
        self.broadcast_reliable(&msg);
    }

    pub fn run(mut self, cmd_rx: mpsc::Receiver<ServerCommand>) {
        let tick_duration = Duration::from_millis(16); // ~1/60th of a second
        let mut last_time = Instant::now();

        println!(
            "[server][{:?}] Server started at {:?}",
            SystemTime::now(),
            self.socket.public_addr()
        );

        loop {
            let now = Instant::now();
            let frame_time = (now - last_time).as_secs_f32();

            // Check for shutdown command
            if let Ok(ServerCommand::Shutdown) = cmd_rx.try_recv() {
                println!("[server] Shutdown requested");
                break;
            }

            // Receive messages from clients
            loop {
                match self.socket.recv() {
                    Ok(Some((msg, src))) => {
                        self.handle_message(msg, src);
                    }
                    Ok(None) => break, // no more packets
                    Err(e) => {
                        if let Err(e) = ignore_would_block(e) {
                            eprintln!("Error receiving message: {}", e);
                        }
                        break;
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

            if self.state == GameState::InGame {
                self.update_players(frame_time);

                // Check if the game has ended
                if let Some(winner_id) = self.update_bullets(frame_time) {
                    self.handle_game_over(winner_id);
                } else {
                    self.broadcast_game_snapshot();
                }
            }

            // Sleep to maintain 30 Hz tick
            last_time = Instant::now();
            let elapsed = last_time.elapsed();
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
            MessageType::StartGame => self.handle_start_game_msg(msg, src),
            MessageType::GameInput => self.handle_game_input(msg, src),
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

                let mut client = Client::new(payload.username.clone());
                if self.state == GameState::InGame {
                    spawn_client_randomly(&mut client, &self.maze);
                }

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

    fn send_ack(
        &mut self,
        addr: SocketAddr,
        msg: Message,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let seq = self.socket.next_sequence();
        let ack_seq = msg.header.sequence;
        let msg = Message::new_ack(seq, ack_seq);
        self.socket.send(addr, &msg)?;
        Ok(())
    }

    fn send_game_info(&mut self, addr: SocketAddr) {
        let game_info = GameInfoPayload {
            game_name: self.game_name.clone(),
            maze: self.maze.clone(),
            target_score: self.target_score,
            host_username: self.host_username.clone(),
            state: self.state.to_string(),
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

    fn handle_start_game_msg(&mut self, msg: Message, src: SocketAddr) {
        game_on_init(&mut self.clients, &self.maze);
        if let Some(requestor) = self.clients.get(&src) {
            if requestor.id == self.host_username {
                println!("handle game start");
                if let Err(_e) = self.send_ack(src, msg) {
                    eprintln!("[server] Failed to send acknowledgement to {}", src);
                }
                let seq = self.socket.next_sequence();
                let msg = Message::new_game_start(seq);
                self.broadcast_reliable(&msg);
            }
        }
        self.state = GameState::InGame;
    }

    fn build_snapshot(&self, sequence: u32) -> Message {
        let players = self.clients
            .iter()
            .map(|c| (c.id.clone(), c.to_snapshot()))
            .collect();

        let bullets = self.bullets
            .iter()
            .map(|b| BulletSnapshot { pos: b.pos.to_tuple() })
            .collect();

        Message::new_game_snapshot(
            sequence, 
            &GameSnapShotPayload { players, bullets }
        )
    }

    fn handle_game_input(&mut self, msg: Message, src: SocketAddr) {
        let inputs = match msg.decode_game_input() {
            Ok(i) => i,
            Err(_) => return,
        };

        let seq = msg.header.sequence;

        let Some(client) = self.clients.get_mut(&src) else {
            return;
        };

        // Drop out-of-order or duplicate inputs
        if seq <= client.last_seq {
            return;
        }

        client.last_seq = seq;
        client.last_input = Some(inputs);
    }

    fn broadcast_game_snapshot(&mut self) {
        let seq = self.socket.next_sequence();
        let msg = self.build_snapshot(seq);

        self.broadcast(&msg);
    }

    fn update_players(&mut self, dt: f32) {
        for client in self.clients.iter_mut() {
            update_player(client, &self.maze, &mut self.bullets, dt);
        }
    }

    pub fn update_bullets(&mut self, dt: f32) ->Option<String> {
        let mut hits = Vec::new();
        let maze = &self.maze;
        let clients = &mut self.clients;

        self.bullets.retain_mut(|bullet| {
            match update_bullet(bullet, maze, clients, dt) {
                BulletStatus::Active => true,
                BulletStatus::HitWall => false,
                BulletStatus::HitPlayer(victim_id) => {
                    hits.push((bullet.owner_id.clone(), victim_id));
                    false
                }
            }
        });

        for (attacker, victim) in hits {
            let score = handle_bullet_hit(&attacker, &victim, &mut self.clients);
            if score >= self.target_score {
                return Some(attacker);
            }
        }

        None
    }

    fn handle_game_over(&mut self, winner_id: String) {
        // 1. Log
        println!("[server] Game Over! Winner: {}", winner_id);

        // 2. Reset Server State
        self.bullets.clear();
        self.state = GameState::Lobby;

        // 3. Notify all clients to switch back to Lobby UI
        let seq = self.socket.next_sequence();
        let msg = Message::new_game_end(seq, &winner_id); 
        self.broadcast_reliable(&msg);

        // 4. Refresh Game Info for all clients (updates their local state to Lobby)
        self.clients.reset_all();
    }
}
