use std::collections::HashMap;
use std::error::Error;
use std::net::SocketAddr;
use std::time::Duration;

use super::{Message, NetSocket, PingManager, now_ms};

/// Represents a connected client
pub struct ClientInfo {
    pub last_seen: u64,
}

/// ServerSocket manages multiple clients over UDP
pub struct ServerSocket {
    socket: NetSocket,
    clients: HashMap<SocketAddr, ClientInfo>,
    ping_manager: PingManager,
    timeout: Duration,
}

impl ServerSocket {
    /// Bind to a local address to accept clients
    pub fn bind(local_addr: &str, timeout: Duration) -> Result<Self, Box<dyn Error>> {
        let socket = NetSocket::bind(local_addr, timeout)?;
        Ok(Self {
            socket,
            clients: HashMap::new(),
            ping_manager: PingManager::new(timeout),
            timeout,
        })
    }

    fn mark_client_alive(&mut self, addr: SocketAddr) {
        self.clients
            .entry(addr)
            .or_insert(ClientInfo {
                last_seen: now_ms(),
            })
            .last_seen = now_ms();
    }

    /// Receive a message from any client
    pub fn recv(&mut self) -> Result<Option<(Message, SocketAddr)>, Box<dyn Error>> {
        match self.socket.recv() {
            Ok((msg, addr)) => {
                self.mark_client_alive(addr);
                Ok(Some((msg, addr)))
            }
            Err(e) => Err(e),
        }
    }

    pub fn next_sequence(&mut self) -> u32 {
        self.socket.next_sequence()
    }

    /// Send a message to a specific client
    pub fn send(&mut self, addr: SocketAddr, msg: &Message) -> Result<usize, Box<dyn Error>> {
        self.socket.send(addr, msg)
    }

    pub fn send_reliable(
        &mut self,
        addr: SocketAddr,
        msg: &Message,
    ) -> Result<usize, Box<dyn Error>> {
        self.socket.send_reliable(addr, msg)
    }

    pub fn resend_pending(&mut self) -> Vec<(u32, SocketAddr)> {
        self.socket.resend_pending()
    }

    /// Broadcast a message to all connected clients
    pub fn broadcast(&mut self, msg: &Message) -> Result<Vec<SocketAddr>, Box<dyn Error>> {
        let addrs: Vec<_> = self.clients.keys().copied().collect();
        let mut failed = Vec::new();
        for addr in addrs {
            if let Err(e) = self.send(addr, msg) {
                eprintln!("Failed to send to {}: {}", addr, e);
                failed.push(addr);
            }
        }
        Ok(failed)
    }

    /// Send a ping to a specific client and return the sequence number
    pub fn send_ping(&mut self, addr: SocketAddr) -> Result<u32, Box<dyn Error>> {
        let seq = self.ping_manager.create_ping(addr);
        let ping_msg = Message::new_ping(seq);
        self.send(addr, &ping_msg)?;
        Ok(seq)
    }

    /// Handle a pong from a client and reset the timeout for that client
    pub fn handle_pong(&mut self, addr: SocketAddr, seq: u32) -> Option<Duration> {
        self.mark_client_alive(addr);
        self.ping_manager.handle_pong(seq, addr)
    }

    pub fn handle_ping(
        &mut self,
        addr: SocketAddr,
        seq: u32,
    ) -> Result<(SocketAddr, Message), Box<dyn Error>> {
        self.mark_client_alive(addr);
        let pong_msg = Message::new_pong(seq);
        Ok((addr, pong_msg))
    }

    pub fn check_ping_timeouts(&mut self) -> Vec<SocketAddr> {
        self.ping_manager.check_timeouts()
    }

    /// Get a list of currently connected clients
    pub fn client_list(&self) -> Vec<SocketAddr> {
        self.clients.keys().copied().collect()
    }

    /// Remove clients that have timed out based on last_seen
    pub fn remove_stale_clients(&mut self) -> Vec<SocketAddr> {
        let now_ms = now_ms();
        let mut removed = Vec::new();

        self.clients.retain(|addr, info| {
            if now_ms.saturating_sub(info.last_seen) > self.timeout.as_millis() as u64 {
                removed.push(*addr);
                false
            } else {
                true
            }
        });

        removed
    }

    pub fn local_addr(&self) -> Result<SocketAddr, Box<dyn Error>> {
        self.socket.local_addr()
    }

    /// Get local socket address
    pub fn local_ip(&self) -> Result<String, Box<dyn Error>> {
        self.socket.local_ip()
    }

    pub fn public_addr(&self) -> Result<String, Box<dyn Error>> {
        let port = self.socket.local_addr()?.port();
        Ok(format!("{}:{}", self.local_ip()?, port))
    }
}

#[cfg(test)]
mod tests {
    use super::super::ClientSocket;
    use super::*;
    use std::net::SocketAddr;
    use std::thread;
    use std::time::Duration;

    #[test]
    fn test_server_socket_creation() {
        let timeout = Duration::from_secs(2);
        let local_addr = "127.0.0.1:0"; // dynamically assigned port

        let server_socket = ServerSocket::bind(local_addr, timeout).unwrap();

        let local_addr = server_socket.local_addr().unwrap();
        println!("Server is bound to: {:?}", local_addr);
        assert!(local_addr.port() > 0);
    }

    #[test]
    fn test_recv_message() {
        let timeout = Duration::from_secs(2);
        let server_addr = "127.0.0.1:8081";
        let client_addr = "127.0.0.1:8082";

        let mut server_socket = ServerSocket::bind(server_addr, timeout).unwrap();
        let mut client_socket = ClientSocket::new(client_addr, server_addr, timeout).unwrap();
        let client_addr_parsed: SocketAddr = client_addr.parse().unwrap();

        // Send ping from client
        let msg = Message::new_ping(1);
        client_socket.send(&msg).unwrap();

        // Poll server for message
        let mut received = None;
        let start = std::time::Instant::now();
        while start.elapsed() < Duration::from_secs(2) {
            if let Some((msg, src)) = server_socket.recv().unwrap() {
                received = Some((msg, src));
                break;
            }
            thread::sleep(Duration::from_millis(10));
        }

        let (msg, src) = received.expect("Did not receive any message in time");

        assert_eq!(src, client_addr_parsed);
        assert!(msg.is_ping(), "Expected a ping message");
    }

    #[test]
    fn test_send_message() {
        let timeout = Duration::from_secs(2);
        let mut server = ServerSocket::bind("127.0.0.1:0", timeout).unwrap();
        let real_server_addr = server.local_addr().unwrap();

        let mut client_socket =
            ClientSocket::new("127.0.0.1:0", &real_server_addr.to_string(), timeout).unwrap();
        let client_real_addr = client_socket.local_addr().unwrap();

        let msg = Message::new_ping(1);
        let sent_bytes = server.send(client_real_addr, &msg).unwrap();
        assert!(sent_bytes > 0, "Server failed to send message");

        // Check that client received the message
        let mut received = None;
        let start = std::time::Instant::now();
        while start.elapsed() < Duration::from_secs(2) {
            if let Some(msg) = client_socket.recv().unwrap() {
                received = Some(msg);
                break;
            }
            thread::sleep(Duration::from_millis(5));
        }
        let received = received.expect("Client did not receive message");
        assert!(received.is_ping());
    }

    #[test]
    fn test_broadcast_message() {
        let timeout = Duration::from_secs(2);
        let mut server = ServerSocket::bind("127.0.0.1:0", timeout).unwrap();
        let real_server_addr = server.local_addr().unwrap();

        let client1 =
            ClientSocket::new("127.0.0.1:0", &real_server_addr.to_string(), timeout).unwrap();
        let client2 =
            ClientSocket::new("127.0.0.1:0", &real_server_addr.to_string(), timeout).unwrap();

        // Register clients manually
        server
            .clients
            .insert(client1.local_addr().unwrap(), ClientInfo { last_seen: 0 });
        server
            .clients
            .insert(client2.local_addr().unwrap(), ClientInfo { last_seen: 0 });

        let msg = Message::new_ping(1);

        // Spawn threads to receive messages
        let c1 = thread::spawn({
            let mut client1 = client1;
            let msg_type = msg.header.msg_type;
            move || {
                let mut received = None;
                let start = std::time::Instant::now();
                while start.elapsed() < Duration::from_secs(1) {
                    match client1.recv() {
                        Ok(Some(msg)) => {
                            received = Some(msg);
                            break;
                        }
                        Ok(None) => { /* ignore messages from other sources */ }
                        Err(_) => {
                            // Ignore errors (like WouldBlock) and retry
                        }
                    }
                    thread::sleep(Duration::from_millis(5));
                }
                let received = received.expect("Client1 did not receive any message in time");
                assert_eq!(received.header.msg_type, msg_type);
            }
        });

        let c2 = thread::spawn({
            let mut client2 = client2;
            let msg_type = msg.header.msg_type;
            move || {
                let mut received = None;
                let start = std::time::Instant::now();
                while start.elapsed() < Duration::from_secs(1) {
                    match client2.recv() {
                        Ok(Some(msg)) => {
                            received = Some(msg);
                            break;
                        }
                        Ok(None) => { /* ignore messages from other sources */ }
                        Err(_) => {
                            // Ignore errors (like WouldBlock) and retry
                        }
                    }
                    thread::sleep(Duration::from_millis(5));
                }
                let received = received.expect("Client2 did not receive any message in time");
                assert_eq!(received.header.msg_type, msg_type);
            }
        });

        // Broadcast message
        server.broadcast(&msg).unwrap();

        c1.join().unwrap();
        c2.join().unwrap();
    }

    #[test]
    fn test_remove_stale_clients() {
        let timeout = Duration::from_secs(1);
        let mut server = ServerSocket::bind("127.0.0.1:0", timeout).unwrap();

        let client_addr = "127.0.0.1:12345".parse().unwrap();
        server.clients.insert(
            client_addr,
            ClientInfo {
                last_seen: now_ms(),
            },
        );

        std::thread::sleep(Duration::from_secs(2));

        let removed = server.remove_stale_clients();
        assert_eq!(removed.len(), 1, "Stale client was not removed");
        assert_eq!(removed[0], client_addr, "Incorrect client was removed");
        assert!(server.clients.is_empty());
    }
}
