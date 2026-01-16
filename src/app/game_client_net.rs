//! # Game Client Network Module
//!
//! Manages client-side networking in a separate thread. Provides thread-safe
//! communication between the game logic (main thread) and the networking layer.
//!
//! Uses `mpsc::channel` for thread-safe message passing:
//! - `GameNetCommand`: Commands from game thread to network thread
//! - `GameNetEvent`: Events from network thread to game thread

use std::net::SocketAddr;
use std::sync::mpsc;
use std::thread;
use std::time::{Duration, Instant};

use crate::app::constants::client;
use fps_net::{ClientSocket, Message, MessageType, ignore_would_block};

/// Commands sent FROM the game logic TO the networking thread.
///
/// These commands request network operations like sending messages or shutting down.
#[derive(Debug)]
pub enum GameNetCommand {
    /// Send an unreliable message.
    Send(Message),
    
    /// Send a reliable message (with retry/acknowledgment).
    SendReliable(Message),
    
    /// Shutdown the networking thread.
    Shutdown,
}

/// Events sent FROM the networking thread TO the game logic.
///
/// These events represent network messages received from the server or
/// connection state changes.
#[derive(Debug)]
pub enum GameNetEvent {
    /// Updated list of connected client IDs.
    ClientList(Vec<String>),
    
    /// Game has started (transition from lobby to game).
    GameStart,
    
    /// Game has ended (includes winner ID).
    GameEnd(String),
    
    /// Game state snapshot from server.
    Snapshot(Message),
    
    /// Chat message received.
    Chat(Message),
    
    /// Connection lost or disconnected.
    Disconnected,
}

/// Handle for interacting with the client networking thread.
///
/// Provides methods to send commands and receive events from the networking thread.
/// The handle can be used from the main thread safely.
#[derive(Debug)]
pub struct GameNetHandle {
    /// Channel sender for sending commands to the network thread.
    pub cmd_tx: mpsc::Sender<GameNetCommand>,
    
    /// Channel receiver for receiving events from the network thread.
    pub evt_rx: mpsc::Receiver<GameNetEvent>,
    
    /// Join handle for the networking thread (for cleanup).
    pub join: thread::JoinHandle<()>,
    
    /// Sequence number generator for messages.
    sequence: u32,
}

impl GameNetHandle {
    /// Shuts down the networking thread gracefully.
    ///
    /// Sends a shutdown command and waits for the thread to finish.
    pub fn shutdown(self) {
        let _ = self.cmd_tx.send(GameNetCommand::Shutdown);
        let _ = self.join.join();
    }

    /// Sends an unreliable message to the server.
    ///
    /// Unreliable messages are not retried if lost. Suitable for high-frequency
    /// messages like game inputs where occasional loss is acceptable.
    ///
    /// # Arguments
    ///
    /// * `msg` - The message to send
    ///
    /// # Returns
    ///
    /// `Ok(())` if the command was sent, error if the channel is closed.
    pub fn send(&self, msg: Message) -> Result<(), Box<dyn std::error::Error>> {
        self.cmd_tx.send(GameNetCommand::Send(msg))?;
        Ok(())
    }

    /// Sends a reliable message to the server.
    ///
    /// Reliable messages are retried until acknowledged. Use for important
    /// messages like chat or game start requests.
    ///
    /// # Arguments
    ///
    /// * `msg` - The message to send
    ///
    /// # Returns
    ///
    /// `Ok(())` if the command was sent, error if the channel is closed.
    pub fn send_reliable(&self, msg: Message) -> Result<(), Box<dyn std::error::Error>> {
        self.cmd_tx.send(GameNetCommand::SendReliable(msg))?;
        Ok(())
    }

    /// Attempts to receive a network event non-blockingly.
    ///
    /// Returns `None` if no events are available. Should be called each frame
    /// to process incoming network messages.
    ///
    /// # Returns
    ///
    /// `Some(event)` if an event is available, `None` otherwise.
    pub fn try_recv(&self) -> Option<GameNetEvent> {
        self.evt_rx.try_recv().ok()
    }

    /// Generates the next sequence number for messages.
    ///
    /// Used to track message ordering and acknowledgments. Sequence numbers
    /// wrap around to 0 after reaching u32::MAX.
    ///
    /// # Returns
    ///
    /// The next sequence number.
    pub fn next_sequence(&mut self) -> u32 {
        let seq = self.sequence;
        self.sequence = self.sequence.wrapping_add(1);
        seq
    }
}

/// Internal networking worker that runs in a separate thread.
///
/// Handles all UDP socket I/O, message encoding/decoding, and ping management.
/// Communicates with the main thread via channels.
struct GameNet {
    socket: ClientSocket,
    _server_addr: SocketAddr,

    cmd_rx: mpsc::Receiver<GameNetCommand>,
    evt_tx: mpsc::Sender<GameNetEvent>,

    last_ping: Instant,
    ping_interval: Duration,
}

impl GameNet {
    fn run(mut self) {
        loop {
            // ---- handle outgoing commands ----
            while let Ok(cmd) = self.cmd_rx.try_recv() {
                match cmd {
                    GameNetCommand::Send(msg) => {
                        let _ = self.socket.send(&msg);
                    }
                    GameNetCommand::SendReliable(msg) => {
                        let _ = self.socket.send_reliable(&msg);
                    }
                    GameNetCommand::Shutdown => {
                        let _ = self.socket.send(&Message::new_disconnect_notice(0));
                        return;
                    }
                }
            }

            // ---- receive network messages ----
            loop {
                match self.socket.recv() {
                    Ok(Some(msg)) => self.handle_message(msg),
                    Ok(None) => break, // no more packets
                    Err(e) => {
                        if ignore_would_block(e).is_err() {
                            let _ = self.evt_tx.send(GameNetEvent::Disconnected);
                            return;
                        }
                        break;
                    }
                }
            }

            // ---- ping ----
            if self.last_ping.elapsed() >= self.ping_interval {
                let _ = self.socket.send_ping();
                self.last_ping = Instant::now();
            }

            // Resend reliable packets
            self.socket.resend_pending();
            
            std::thread::sleep(Duration::from_millis(client::NET_THREAD_SLEEP_MS));
        }
    }

    fn handle_message(&mut self, msg: Message) {
        //println!("handling msg {:?}", msg.header);
        match msg.header.msg_type {
            MessageType::Pong => {
                let _rtt = self.socket.handle_pong(msg.header.sequence);
            }

            MessageType::ChatMessage => {
                let _ = self.evt_tx.send(GameNetEvent::Chat(msg));
            }

            MessageType::ClientList => match msg.decode_client_list() {
                Ok(payload) => {
                    let ids = payload.clients;

                    let _ = self.send_ack(msg);
                    let _ = self.evt_tx.send(GameNetEvent::ClientList(ids));
                }
                Err(e) => {
                    eprintln!("[gamenet] Failed to decode ClientList: {}", e);
                }
            },

            MessageType::StartGame => {
                let _ = self.send_ack(msg);
                let _ = self.evt_tx.send(GameNetEvent::GameStart);
            }

            MessageType::GameSnapShot => {
                let _ = self.evt_tx.send(GameNetEvent::Snapshot(msg));
            }

            MessageType::GameEnd => {
                let payload = msg.decode_game_end().unwrap();
                let _ = self.evt_tx.send(GameNetEvent::GameEnd(payload.winner));
                let _ = self.send_ack(msg);
            }
           
            _ => {}
        }
    }

    fn send_ack(&mut self, msg: Message) -> Result<(), Box<dyn std::error::Error>> {
        let seq = self.socket.next_sequence();
        let ack_seq = msg.header.sequence;
        let msg = Message::new_ack(seq, ack_seq);
        self.socket.send(&msg)?;
        Ok(())
    }
}

/// Starts the client networking thread and returns a handle.
///
/// Creates a new thread that handles all network communication for the client.
/// The thread runs until it receives a shutdown command.
///
/// # Arguments
///
/// * `socket` - The client socket to use for communication
/// * `_server_addr` - The server address (currently unused but kept for future use)
///
/// # Returns
///
/// A `GameNetHandle` for interacting with the networking thread.
pub fn start_game_net(socket: ClientSocket, _server_addr: SocketAddr) -> GameNetHandle {
    let (cmd_tx, cmd_rx) = mpsc::channel();
    let (evt_tx, evt_rx) = mpsc::channel();

    let net = GameNet {
        socket,
        _server_addr,
        cmd_rx,
        evt_tx,
        last_ping: Instant::now(),
        ping_interval: Duration::from_secs(client::PING_INTERVAL_SECONDS),
    };

    let join = thread::spawn(move || {
        if let Err(e) = std::panic::catch_unwind(|| net.run()) {
            eprintln!("client_net thread crashed: {:?}", e);
        }
    });

    GameNetHandle {
        cmd_tx,
        evt_rx,
        join,
        sequence: 0,
    }
}
