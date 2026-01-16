//! # Client Module
//!
//! Manages connected clients (players) on the server side. Provides data structures
//! for tracking client state, scores, positions, and status effects like invincibility.

use std::collections::HashMap;
use std::net::SocketAddr;
use std::time::Instant;

use fps_net::message::{GameInputPayload, PlayerSnapshot};

use super::Pos;

/// Represents the current status of a client.
///
/// Tracks special states like invincibility periods after being hit.
#[derive(Debug, Clone, PartialEq)]
pub enum ClientStatus {
    /// Normal state - client can be damaged normally.
    Normal,
    /// Invincible state - client cannot be damaged.
    /// Contains the remaining invincibility time in seconds.
    Invincible(f32),
}

/// Represents a player connected to the game.
///
/// Stores all state information for a single player including position, score,
/// input history, and status effects. This is the server's authoritative representation
/// of a player.
#[derive(Debug, Clone)]
pub struct Client {
    /// Unique identifier (username) for this client.
    pub id: String,
    
    /// Current score of this client.
    pub score: u32,
    
    /// Current position and rotation in the game world.
    pub pos: Pos,
    
    /// Most recent input received from the client.
    pub last_input: Option<GameInputPayload>,
    
    /// Sequence number of the last processed input message.
    pub last_seq: u32,

    /// Current status (normal or invincible).
    pub status: ClientStatus,
    
    /// Timestamp of the last shot fired (for cooldown management).
    pub last_shot_time: Instant,
}

impl Client {
    /// Creates a new client with default initial state.
    ///
    /// # Arguments
    ///
    /// * `id` - Unique identifier (username) for the client
    ///
    /// # Returns
    ///
    /// A new `Client` instance with zero score, default position, and normal status.
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

    /// Resets client state for a new game.
    ///
    /// Resets score to zero, clears invincibility status, and resets position
    /// to default. Called when starting a new game round.
    pub fn reset(&mut self) {
        self.score = 0;
        self.status = ClientStatus::Normal;
        self.pos.reset();
    }

    /// Checks if the client is currently invincible.
    ///
    /// # Returns
    ///
    /// `true` if the client is in invincible state, `false` otherwise.
    pub fn is_invincible(&self) -> bool {
        matches!(self.status, ClientStatus::Invincible(_))
    }

    /// Converts the client state to a network snapshot.
    ///
    /// Creates a `PlayerSnapshot` that can be serialized and sent to clients
    /// for rendering. This is the server's authoritative player state.
    ///
    /// # Returns
    ///
    /// A `PlayerSnapshot` containing the current player state.
    pub fn to_snapshot(&self) -> PlayerSnapshot {
        PlayerSnapshot {
            pos: self.pos.to_tuple(),
            score: self.score,
            is_invincible: self.is_invincible(),
        }
    }
}

/// Manages a collection of connected clients.
///
/// Provides methods for adding, removing, and querying clients by their
/// network address. Used by the server to track all connected players.
#[derive(Debug, Clone)]
pub struct ClientList {
    /// Internal map of socket addresses to client data.
    clients: HashMap<SocketAddr, Client>,
}

impl ClientList {
    /// Creates a new empty client list.
    ///
    /// # Returns
    ///
    /// A new `ClientList` with no clients.
    pub fn new() -> Self {
        Self {
            clients: HashMap::new(),
        }
    }

    /// Returns the number of connected clients.
    ///
    /// # Returns
    ///
    /// The number of clients currently in the list.
    pub fn len(&self) -> usize {
        self.clients.len()
    }

    /// Adds a new client to the list.
    ///
    /// The client will only be added if:
    /// - No client exists with the same socket address
    /// - No client exists with the same ID (username)
    ///
    /// # Arguments
    ///
    /// * `client` - The client to add
    /// * `addr` - The socket address associated with this client
    ///
    /// # Returns
    ///
    /// `true` if the client was added, `false` if a duplicate was found.
    pub fn add_client(&mut self, client: Client, addr: SocketAddr) -> bool {
        if self.clients.contains_key(&addr) || self.clients.values().any(|c| c.id == client.id) {
            return false;
        }

        self.clients.insert(addr, client);
        true
    }

    /// Removes a client from the list by socket address.
    ///
    /// # Arguments
    ///
    /// * `addr` - The socket address of the client to remove
    ///
    /// # Returns
    ///
    /// The removed `Client` if found, `None` otherwise.
    pub fn remove_client(&mut self, addr: &SocketAddr) -> Option<Client> {
        self.clients.remove(addr)
    }

    /// Gets an immutable reference to a client by socket address.
    ///
    /// # Arguments
    ///
    /// * `addr` - The socket address of the client
    ///
    /// # Returns
    ///
    /// A reference to the client if found, `None` otherwise.
    pub fn get(&self, addr: &SocketAddr) -> Option<&Client> {
        self.clients.get(addr)
    }

    /// Gets a mutable reference to a client by socket address.
    ///
    /// # Arguments
    ///
    /// * `addr` - The socket address of the client
    ///
    /// # Returns
    ///
    /// A mutable reference to the client if found, `None` otherwise.
    pub fn get_mut(&mut self, addr: &SocketAddr) -> Option<&mut Client> {
        self.clients.get_mut(addr)
    }

    /// Resets all clients for a new game round.
    ///
    /// Calls `reset()` on each client, clearing scores, positions, and status effects.
    pub fn reset_all(&mut self) {
        for client in self.clients.values_mut() {
            client.reset();
        }
    }

    /// Returns a vector of references to all clients.
    ///
    /// # Returns
    ///
    /// A vector containing references to all clients in the list.
    pub fn all_clients(&self) -> Vec<&Client> {
        self.clients.values().collect()
    }

    /// Returns an iterator over all clients.
    ///
    /// # Returns
    ///
    /// An iterator yielding immutable references to clients.
    pub fn iter(&self) -> impl Iterator<Item = &Client> {
        self.clients.values()
    }

    /// Returns a mutable iterator over all clients.
    ///
    /// # Returns
    ///
    /// An iterator yielding mutable references to clients.
    pub fn iter_mut(&mut self) -> impl Iterator<Item = &mut Client> {
        self.clients.values_mut()
    }
}
