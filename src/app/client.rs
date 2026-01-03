use std::collections::HashMap;
use std::net::SocketAddr;

/// Represents a player connected to the game
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Client {
    pub id: String, // Username or unique player ID
    pub score: u32, // Current score
}

impl Client {
    pub fn new(id: String) -> Self {
        Self { id, score: 0 }
    }

    /// Reset score and ready status (e.g., new game)
    pub fn reset(&mut self) {
        self.score = 0;
    }
}

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

    /// Return a list of addresses of all clients
    pub fn all_addresses(&self) -> Vec<SocketAddr> {
        self.clients.keys().copied().collect()
    }

    /// Update a client's score
    pub fn update_score(&mut self, addr: &SocketAddr, score: u32) {
        if let Some(client) = self.clients.get_mut(addr) {
            client.score = score;
        }
    }
}
