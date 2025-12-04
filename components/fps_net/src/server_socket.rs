use std::collections::HashMap;
use std::net::SocketAddr;
use std::time::{Duration, SystemTime, UNIX_EPOCH};
use std::io::{Result};

use super::{NetSocket, Message, PingManager};

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
    pub fn bind(local_addr: &str, timeout: Duration) -> Result<Self> {
        let socket = NetSocket::bind(local_addr, timeout)?;
        Ok(Self {
            socket,
            clients: HashMap::new(),
            ping_manager: PingManager::new(timeout),
            timeout,
        })
    }

    /// Receive a message from any client
    pub fn recv(&mut self) -> Result<(Message, SocketAddr)> {
        let (msg, addr) = self.socket.recv()?;

        // Update last_seen or register new client
        let entry = self.clients.entry(addr)
            .or_insert_with(|| ClientInfo { last_seen: msg.header.timestamp_ms });
        entry.last_seen = msg.header.timestamp_ms;

        Ok((msg, addr))
    }

    /// Send a message to a specific client
    pub fn send(&mut self, addr: SocketAddr, msg: &Message) -> Result<usize> {
        self.socket.send(addr, msg)
    }
    
    pub fn resend_pending(&mut self) -> Result<()> {
        self.socket.resend_pending()
    }
    
    /// Send a ping to a specific client and return the sequence number
    pub fn send_ping(&mut self, addr: SocketAddr) -> Result<u32> {
        let seq = self.ping_manager.create_ping(addr);
        let ping_msg = Message::new_ping(seq);
        self.send(addr, &ping_msg)?;
        Ok(seq)
    }

    /// Handle a pong from a client and reset the timeout for that client
    pub fn handle_pong(&mut self, addr: SocketAddr, seq: u32) -> Option<Duration> {
        self.ping_manager.handle_pong(seq, addr)
    }

    /// Broadcast a message to all connected clients
    pub fn broadcast(&mut self, msg: &Message) -> Result<Vec<SocketAddr>> {
      let clients = self.clients.keys().cloned().collect::<Vec<_>>();
        let mut failed = Vec::new();

        for addr in clients {
            if let Err(e) = self.send(addr, msg) {
                eprintln!("Failed to send to {}: {}", addr, e);
                failed.push(addr);
            }
        }

        Ok(failed)
    }

    pub fn handle_ping(&mut self, addr: SocketAddr, seq: u32) -> Result<()> {
        let pong_msg = Message::new_pong(seq);
        self.send(addr, &pong_msg)?;
        Ok(())
    }

    /// Remove clients that have timed out based on last_seen
    pub fn remove_stale_clients(&mut self) -> Vec<SocketAddr> {
        let now_ms = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_millis() as u64;

        let mut removed = Vec::new();

        // Retain clients whose last_seen is within the timeout
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

    pub fn next_sequence(&mut self) -> u32 {
        self.socket.next_sequence()
    }

    /// Get a list of currently connected clients
    pub fn client_list(&self) -> Vec<SocketAddr> {
        self.clients.keys().cloned().collect()
    }

    pub fn check_ping_timeouts(&mut self) -> Vec<SocketAddr> {
        self.ping_manager.check_timeouts()
    }

    pub fn local_addr(&self) -> Result<SocketAddr> {
        self.socket.local_addr()
    }

}

#[cfg(test)]
mod tests {
    use super::*;
    use super::super::{ClientSocket, now_ms};
    use std::sync::{Arc, Mutex};
    use std::thread;
    use std::time::Duration;
    use std::net::SocketAddr;

    #[test]
    fn test_server_socket_creation() {
        let timeout = Duration::from_secs(2);
        let local_addr = "127.0.0.1:0"; // Use a dynamically assigned local port

        // Create ServerSocket
        let server_socket = ServerSocket::bind(local_addr, timeout).unwrap();

        // Ensure that the server is listening on the dynamically assigned address
        println!("Server is bound to: {:?}", server_socket.socket.local_addr().unwrap());
        assert!(server_socket.socket.local_addr().unwrap().port() > 0);
    }

    #[test]
    fn test_recv_message() {
        let timeout = Duration::from_secs(2);
        let server_addr = "127.0.0.1:8081";
        let client_addr = "127.0.0.1:8082";

        let server_socket = Arc::new(Mutex::new(
            ServerSocket::bind(server_addr, timeout).unwrap()
        ));

        let client_socket = Arc::new(Mutex::new(
            ClientSocket::new(client_addr, server_addr, timeout).unwrap()
        ));

        // Start server thread
        let server_socket_clone = server_socket.clone();
        let handle = thread::spawn(move || {
            let mut server_socket = server_socket_clone.lock().unwrap();
            let (msg, src) = server_socket.recv().unwrap();

            assert_eq!(src, client_addr.parse::<SocketAddr>().unwrap());

            // This SHOULD FAIL because msg is Ping
            assert!(msg.is_ping(), "Expected a ping message");
        });

        // Send ping
        {
            let mut client_socket = client_socket.lock().unwrap();
            let msg = Message::new_ping(1);
            client_socket.send(&msg).unwrap();
        }

        // Wait for server thread and propagate failure
        handle.join().unwrap();

        thread::sleep(Duration::from_millis(100));
    }

    /// Test sending a message to a client
    #[test]
    fn test_send_message() {
        let timeout = Duration::from_secs(2);
        let server_addr = "127.0.0.1:0";
        let client_addr = "127.0.0.1:0";

        let mut server = ServerSocket::bind(server_addr, timeout).unwrap();
        let real_server_addr = server.local_addr().unwrap();

        let client_socket = super::super::ClientSocket::new(
            client_addr, &real_server_addr.to_string(), timeout).unwrap();
        let client_real_addr = client_socket.local_addr().unwrap();

        let msg = Message::new_ping(1);
        let sent_bytes = server.send(client_real_addr, &msg).unwrap();
        assert!(sent_bytes > 0, "Server failed to send message");
    }

    /// Test broadcasting to multiple clients
    #[test]
    fn test_broadcast_message() {
        let timeout = Duration::from_secs(2);
        let server_addr = "127.0.0.1:0";

        let mut server = ServerSocket::bind(server_addr, timeout).unwrap();
        let real_server_addr = server.socket.local_addr().unwrap();

        // Create two mock clients
        let client1 = super::super::ClientSocket::new(
            "127.0.0.1:0", &real_server_addr.to_string(), timeout).unwrap();
        let client2 = super::super::ClientSocket::new(
            "127.0.0.1:0", &real_server_addr.to_string(), timeout).unwrap();

        let msg = Message::new_ping(1);

        // Register clients manually in server for testing broadcast
        server.clients.insert(client1.local_addr().unwrap(), super::ClientInfo { last_seen: 0 });
        server.clients.insert(client2.local_addr().unwrap(), super::ClientInfo { last_seen: 0 });

        // Spawn threads to receive on clients
        let c1 = thread::spawn({
            let mut client1 = client1;
            move || {
                let received = client1.recv().unwrap().unwrap();
                assert_eq!(received.header.msg_type, msg.header.msg_type);
            }
        });

        let c2 = thread::spawn({
            let mut client2 = client2;
            move || {
                let received = client2.recv().unwrap().unwrap();
                assert_eq!(received.header.msg_type, msg.header.msg_type);
            }
        });

        // Broadcast the message
        server.broadcast(&msg).unwrap();

        // Wait for clients to receive
        c1.join().unwrap();
        c2.join().unwrap();
    }

    #[test]
    fn test_remove_stale_clients() {
        let timeout = Duration::from_secs(1);
        let server_addr = "127.0.0.1:0";

        // Create server
        let mut server = ServerSocket::bind(server_addr, timeout).unwrap();

        // Simulate a client sending a message to register it
        let client_addr = "127.0.0.1:12345".parse().unwrap();

        // Manually update last_seen for testing
        server.clients.insert(client_addr, ClientInfo { last_seen: now_ms() });

        // Ensure client is in the list
        assert_eq!(server.clients.len(), 1);

        // Wait longer than the timeout
        std::thread::sleep(Duration::from_secs(2));

        // Remove stale clients
        let removed = server.remove_stale_clients();

        // Client should have been removed
        assert_eq!(removed.len(), 1, "Stale client was not removed");
        assert_eq!(removed[0], client_addr, "Incorrect client was removed");

        // Clients list should now be empty
        assert!(server.clients.is_empty());
    }

}
