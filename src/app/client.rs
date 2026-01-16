use std::collections::HashMap;
use std::net::SocketAddr;
use std::time::Instant;

use fps_net::message::{GameInputPayload, PlayerSnapshot};

use super::Pos;

#[derive(Debug, Clone, PartialEq)]
pub enum ClientStatus {
    Normal,
    Invincible(f32), // Stores remaining invincibility time in seconds
}

/// Represents a player connected to the game
#[derive(Debug, Clone)]
pub struct Client {
    pub id: String, // Username or unique player ID
    pub score: u32, // Current score
    pub pos: Pos,
    
    pub last_input: Option<GameInputPayload>,
    pub last_seq: u32,

    pub status: ClientStatus,
    pub last_shot_time: Instant,
}

impl Client {
    pub fn new(id: String) -> Self {
        Self {
            id,
            score: 0,
            pos: Pos::default(),
            last_input: None,
            last_seq: 0,
            status: ClientStatus::Normal,
            last_shot_time: Instant::now(),
        }
    }

    /// Reset score and ready status (e.g., new game)
    pub fn reset(&mut self) {
        self.score = 0;
        self.status = ClientStatus::Normal;
        self.pos.reset();
    }

    pub fn is_invincible(&self) -> bool {
        matches!(self.status, ClientStatus::Invincible(_))
    }

    pub fn to_snapshot(&self) -> PlayerSnapshot {PlayerSnapshot { 
            pos: self.pos.to_tuple(),
            score: self.score,
            // Map internal ClientStatus to the network-ready PlayerStatus
            is_invincible: self.is_invincible(),
        }
    }
}

#[derive(Debug, Clone)]
/// Manages a list of connected clients
pub struct ClientList {
    clients: HashMap<SocketAddr, Client>,
}

impl ClientList {
    pub fn new() -> Self {
        Self {
            clients: HashMap::new(),
        }
    }

    pub fn len(&self) -> usize {
        self.clients.len()
    }

    /// Add a new client if it doesn't exist
    pub fn add_client(&mut self, client: Client, addr: SocketAddr) -> bool {
        if self.clients.contains_key(&addr) || self.clients.values().any(|c| c.id == client.id) {
            return false;
        }

        self.clients.insert(addr, client);
        true
    }

    /// Remove a client by address
    pub fn remove_client(&mut self, addr: &SocketAddr) -> Option<Client> {
        self.clients.remove(addr)
    }

    /// Get a reference to a client by address
    pub fn get(&self, addr: &SocketAddr) -> Option<&Client> {
        self.clients.get(addr)
    }

    /// Get a mutable reference to a client
    pub fn get_mut(&mut self, addr: &SocketAddr) -> Option<&mut Client> {
        self.clients.get_mut(addr)
    }

    /// Reset all clients (e.g., new game start)
    pub fn reset_all(&mut self) {
        for client in self.clients.values_mut() {
            client.reset();
        }
    }

    /// Return a list of all clients
    pub fn all_clients(&self) -> Vec<&Client> {
        self.clients.values().collect()
    }

    pub fn iter(&self) -> impl Iterator<Item = &Client> {
        self.clients.values()
    }
    
    pub fn iter_mut(&mut self) -> impl Iterator<Item = &mut Client> {
        self.clients.values_mut()
    }
}
