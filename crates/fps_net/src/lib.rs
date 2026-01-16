//! # `fps_net` - Network Communication Library
//!
//! A UDP-based networking library for multiplayer game communication with reliable message delivery,
//! connection management, and ping/latency tracking.
//!
//! ## Features
//!
//! - **UDP Socket Abstraction**: Low-level UDP socket wrapper with non-blocking I/O
//! - **Reliable Message Delivery**: Automatic retransmission for reliable messages
//! - **Connection Management**: Client and server socket abstractions
//! - **Ping/Latency Tracking**: Built-in ping/pong system for connection health monitoring
//! - **Message Serialization**: Binary message encoding/decoding using bincode
//!
//! ## Architecture
//!
//! The crate is organized into several modules:
//!
//! - [`client_socket`]: Client-side socket for connecting to servers
//! - [`server_socket`]: Server-side socket for managing multiple clients
//! - [`net_socket`]: Low-level UDP socket wrapper
//! - [`message`]: Message types and serialization
//! - [`protocol`]: Network protocol definitions
//! - [`ping`]: Ping/pong and latency tracking
//! - [`util`]: Utility functions
//!
//! ## Example: Client Connection
//!
//! ```rust,no_run
//! use fps_net::ClientSocket;
//! use std::time::Duration;
//!
//! let mut client = ClientSocket::new(
//!     "0.0.0.0:0",
//!     "127.0.0.1:9000",
//!     Duration::from_secs(5),
//! )?;
//!
//! let msg = fps_net::Message::new_ping(client.next_sequence());
//! client.send(&msg)?;
//! ```
//!
//! ## Example: Server Setup
//!
//! ```rust,no_run
//! use fps_net::ServerSocket;
//! use std::time::Duration;
//!
//! let mut server = ServerSocket::bind("0.0.0.0:9000", Duration::from_secs(5))?;
//!
//! if let Ok(Some((msg, addr))) = server.recv() {
//!     // Handle message from client at addr
//! }
//! ```

pub mod client_socket;
pub mod message;
pub mod net_socket;
pub mod ping;
pub mod protocol;
pub mod server_socket;
pub mod util;

pub use client_socket::ClientSocket;
pub use message::{Message, encode_payload};
pub use net_socket::NetSocket;
pub use ping::PingManager;
pub use protocol::{MessageHeader, MessageType};
pub use server_socket::ServerSocket;
pub use util::{ignore_would_block, now_ms};

