use std::net::SocketAddr;
use std::sync::mpsc;
use std::thread;
use std::time::{Duration, Instant};

use fps_net::{ClientSocket, Message, MessageType, ignore_would_block};

/// ===== Commands sent FROM Game TO GameNet =====
#[derive(Debug)]
pub enum GameNetCommand {
    Send(Message),
    Shutdown,
}

/// ===== Events sent FROM GameNet TO Game =====
#[derive(Debug)]
pub enum GameNetEvent {
    ClientList(Vec<String>),
    Snapshot(Message),
    Chat(Message),
    Disconnected,
}

#[derive(Debug)]
pub struct GameNetHandle {
    pub cmd_tx: mpsc::Sender<GameNetCommand>,
    pub evt_rx: mpsc::Receiver<GameNetEvent>,
    pub join: thread::JoinHandle<()>,
    sequence: u32,
}

impl GameNetHandle {
    pub fn shutdown(self) {
        let _ = self.cmd_tx.send(GameNetCommand::Shutdown);
        let _ = self.join.join();
    }

    pub fn send(&self, msg: Message) -> Result<(), Box<dyn std::error::Error>> {
        self.cmd_tx.send(GameNetCommand::Send(msg))?;
        Ok(())
    }

    pub fn try_recv(&self) -> Option<GameNetEvent> {
        self.evt_rx.try_recv().ok()
    }

    /// Generate the next sequence number
    pub fn next_sequence(&mut self) -> u32 {
        let seq = self.sequence;
        self.sequence = self.sequence.wrapping_add(1);
        seq
    }
}

/// Internal networking worker (lives in thread)
struct GameNet {
    socket: ClientSocket,
    server_addr: SocketAddr,

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
                    GameNetCommand::Shutdown => {
                        let _ = self.socket.send(&Message::new_disconnect_notice(0));
                        return;
                    }
                }
            }

            // ---- receive network messages ----
            match self.socket.recv() {
                Ok(Some(msg)) => self.handle_message(msg),
                Ok(None) => {}
                Err(e) => {
                    if ignore_would_block(e).is_err() {
                        let _ = self.evt_tx.send(GameNetEvent::Disconnected);
                        return;
                    }
                }
            }

            // ---- ping ----
            if self.last_ping.elapsed() >= self.ping_interval {
                let _ = self.socket.send_ping();
                self.last_ping = Instant::now();
            }

            std::thread::sleep(Duration::from_millis(16));
        }
    }

    fn handle_message(&mut self, msg: Message) {
        println!("handling msg {:?}", msg.header);
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

                    let _ = self.evt_tx.send(GameNetEvent::ClientList(ids));
                }
                Err(e) => {
                    eprintln!("[gamenet] Failed to decode ClientList: {}", e);
                }
            },

            /*
                       MessageType::GameSnapshot => {
                           let _ = self.evt_tx.send(GameNetEvent::Snapshot(msg));
                       }
            */
            _ => {}
        }
    }
}

/// ===== Public constructor =====
pub fn start_game_net(socket: ClientSocket, server_addr: SocketAddr) -> GameNetHandle {
    let (cmd_tx, cmd_rx) = mpsc::channel();
    let (evt_tx, evt_rx) = mpsc::channel();

    let net = GameNet {
        socket,
        server_addr,
        cmd_rx,
        evt_tx,
        last_ping: Instant::now(),
        ping_interval: Duration::from_secs(1),
    };

    let join = thread::spawn(move || net.run());

    GameNetHandle {
        cmd_tx,
        evt_rx,
        join,
        sequence: 0,
    }
}
