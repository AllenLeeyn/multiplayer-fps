use std::net::SocketAddr;
use std::time::Duration;
use std::io::{Result, ErrorKind};

use super::{NetSocket, Message, PingManager};

/// ClientSocket represents a client connection to a server.
pub struct ClientSocket {
    socket: NetSocket,
    server_addr: SocketAddr,
    ping_manager: PingManager,
}

impl ClientSocket {
    /// Create a new ClientSocket bound to a local port, connecting to a server address
    pub fn new(local_addr: &str, server_addr: &str, ping_timeout: Duration) -> Result<Self> {
        let socket = NetSocket::bind(local_addr, ping_timeout)?;
        let server_addr = server_addr.parse().expect("Invalid server address");

        Ok(Self {
            socket,
            server_addr,
            ping_manager: PingManager::new(ping_timeout),
        })
    }

    /// Receive a message from any source
    pub fn recv(&mut self) -> Result<Option<Message>> {
       match self.socket.recv() {
            Ok((msg, src)) => {
                // Check if the message is from the expected source (server)
                if src == self.server_addr {
                    Ok(Some(msg))  // Return the message if it's from the server
                } else {
                    Ok(None)  // Ignore if it's from any other source
                }
            }
            Err(e) if e.kind() == ErrorKind::WouldBlock => {
                Ok(None)
            }
            Err(e) => return Err(e), // timeout or would-block
        }
    }

    pub fn next_sequence(&mut self) -> u32 {
        self.socket.next_sequence()
    }

    /// Send a message to the server
    pub fn send(&mut self, msg: &Message) -> Result<usize> {
        self.socket.send(self.server_addr, msg)
    }

    pub fn send_reliable(&mut self, msg: &Message) -> Result<usize> {
        self.socket.send_reliable(self.server_addr, msg)
    }

    pub fn resend_pending(&mut self) -> Result<()> {
        self.socket.resend_pending()
    }

    /// Send a ping and return the sequence number
    pub fn send_ping(&mut self) -> Result<u32> {
        let seq = self.ping_manager.create_ping(self.server_addr);
        let ping_msg = Message::new_ping(seq);
        self.send(&ping_msg)?;
        Ok(seq)
    }

    /// Handle a pong message and return RTT if available
    pub fn handle_pong(&mut self, seq: u32) -> Option<Duration> {
        self.ping_manager.handle_pong(seq, self.server_addr)
    }

    pub fn handle_ping(&self, seq: u32) -> Result<Message> {
        let pong_msg = Message::new_pong(seq);
        Ok(pong_msg)
    }

    /// Check for ping timeouts
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
    use super::super::encode_payload;
    use std::thread;
    use std::time::Duration;
    use std::sync::{Arc, Mutex};

    /// Mock server for testing purposes
    fn mock_server(server_addr: Arc<SocketAddr>, _client_socket: Arc<Mutex<ClientSocket>>) {
        // This simulates a simple server that always sends a pong back
        thread::spawn(move || {
            let timeout = Duration::from_secs(2);
            let mut mock_server_socket = NetSocket::bind(&server_addr.to_string(), timeout).unwrap();
            
            loop {
                match mock_server_socket.recv() {
                    Ok((msg, src)) => {
                        // Handle reliable messages
                        if msg.is_reliable() {
                            let ack = Message::new_ack(msg.header.sequence, msg.header.sequence);
                            mock_server_socket.send(src, &ack).unwrap();
                            continue;
                        }

                        // Handle ping messages
                        if msg.is_ping() {
                            let pong_msg = Message::new_pong(msg.header.sequence);
                            mock_server_socket.send(src, &pong_msg).unwrap();
                            continue;
                        }
                    }
                    Err(e) => {
                        thread::sleep(Duration::from_millis(10));
                        eprintln!("Error receiving message: {}", e);
                    }
                }
            }
        });
    }

    #[test]
    fn test_client_socket_creation() {
        let timeout = Duration::from_secs(2);
        let local_addr = "127.0.0.1:0";
        let server_addr = "127.0.0.1:12345";

        let client_socket = ClientSocket::new(local_addr, server_addr, timeout).unwrap();
        assert_eq!(client_socket.server_addr, "127.0.0.1:12345".parse().unwrap());
    }

    #[test]
    fn test_send_message() {
        let timeout = Duration::from_secs(2);
        let local_addr = "127.0.0.1:0";
        let server_addr = "127.0.0.1:12345";

        let client_socket = Arc::new(Mutex::new(ClientSocket::new(local_addr, server_addr, timeout).unwrap()));
        let server_addr = Arc::new("127.0.0.1:12345".parse::<SocketAddr>().unwrap());

        // Start the mock server
        mock_server(server_addr.clone(), client_socket.clone());

        // Give the server time to start
        thread::sleep(Duration::from_millis(100));

        // Create a message
        let msg = Message::new_ping(1);
        let mut client_socket = client_socket.lock().unwrap();

        // Send the message
        let sent_size = client_socket.send(&msg).unwrap();
        assert!(sent_size > 0, "Message was not sent properly");
    }

    #[test]
    fn test_send_reliable_message() {
        let timeout = Duration::from_secs(2);
        let local_addr = "127.0.0.1:0";  // Dynamically assigned local address
        let server_addr = "127.0.0.1:22345";  // The server's address

        // Initialize the ClientSocket with the given local and server addresses
        let client_socket = Arc::new(Mutex::new(
            ClientSocket::new(local_addr, server_addr, timeout).unwrap()));
        let server_addr = Arc::new(server_addr.parse::<SocketAddr>().unwrap());

        // Start the mock server to handle incoming messages
        mock_server(server_addr.clone(), client_socket.clone());

        // Give the server a moment to start up
        thread::sleep(Duration::from_millis(100));

        // Create a message with custom content (e.g., some payload data)
        let content = "Hello, Server! This is a reliable message.";
        let payload_obj = encode_payload(&content).unwrap();
        let msg = Message::new_reliable(1, &payload_obj);  // Assuming Message::new_with_payload is available

        let mut client_socket = client_socket.lock().unwrap();

        // Send the reliable message
        let sent_size = client_socket.send_reliable(&msg).unwrap();
        assert!(sent_size > 0, "Reliable message was not sent properly");

        // Simulate receiving the message on the server side (mock server will handle this)
        let start = std::time::Instant::now();
        let received_msg = loop {
            match client_socket.recv() {
                Ok(Some(msg)) => break msg,
                Ok(None) => { /* ignore unexpected source */ }
                Err(e) if e.kind() == std::io::ErrorKind::WouldBlock => {
                    if start.elapsed() > Duration::from_secs(1) {
                        panic!("Timed out waiting for ACK");
                    }
                    thread::sleep(Duration::from_millis(10));
                    continue;
                }
                Err(e) => panic!("Unexpected recv error: {:?}", e),
            }
        };

        // Check that the received message has the same content as the sent one
        assert!(received_msg.is_ack(), "Expected ACK from server");
        assert_eq!(received_msg.header.sequence, msg.header.sequence, "ACK should match the reliable message sequence");

        // Check ACK payload: it should contain the acknowledged sequence number
        let ack_seq = received_msg.decode_payload::<u32>().unwrap();

        assert_eq!(
            ack_seq,
            msg.header.sequence,
            "ACK is not acknowledging the correct message"
        );
    }

    #[test]
    fn test_send_ping_and_receive_pong() {
        let timeout = Duration::from_secs(2);
        let local_addr = "127.0.0.1:0";
        let server_addr = "127.0.0.1:12345";

        let client_socket = Arc::new(Mutex::new(ClientSocket::new(local_addr, server_addr, timeout).unwrap()));
        let server_addr = Arc::new("127.0.0.1:12345".parse::<SocketAddr>().unwrap());

        // Start mock server
        mock_server(server_addr.clone(), client_socket.clone());

        thread::sleep(Duration::from_millis(100));

        let mut client_socket = client_socket.lock().unwrap();

        // Send ping
        let ping_seq = client_socket.send_ping().unwrap();
        println!("Sent ping with sequence: {}", ping_seq);

        // Wait for pong from server
        let start = std::time::Instant::now();
        let pong_msg = loop {
            match client_socket.recv() {
                Ok(Some(msg)) if msg.is_pong() => break msg,
                Ok(_) => continue, // Ignore other packets
                Err(e) if e.kind() == std::io::ErrorKind::WouldBlock => {
                    if start.elapsed() > Duration::from_secs(1) {
                        panic!("Timed out waiting for pong");
                    }
                    thread::sleep(Duration::from_millis(5));
                }
                Err(e) => panic!("recv error: {:?}", e),
            }
        };

        // Now handle pong
        let rtt = client_socket.handle_pong(pong_msg.header.sequence);
        println!("RTT: {:?}", rtt.unwrap());
        assert!(rtt.is_some(), "Failed to compute RTT after receiving pong");
    }

    #[test]
    fn test_ping_timeout() {
        let timeout = Duration::from_secs(2);
        let local_addr = "127.0.0.1:0";
        let server_addr = "127.0.0.1:12345";

        let client_socket = Arc::new(Mutex::new(ClientSocket::new(local_addr, server_addr, timeout).unwrap()));
        let server_addr = Arc::new("127.0.0.1:12345".parse::<SocketAddr>().unwrap());

        // Start the mock server
        mock_server(server_addr.clone(), client_socket.clone());

        // Give the server time to start
        thread::sleep(Duration::from_millis(100));

        let mut client_socket = client_socket.lock().unwrap();

        // Simulate sending a ping
        let ping_seq = client_socket.send_ping().unwrap();
        println!("Sent ping with sequence: {}", ping_seq);

        // Wait for some time to simulate a timeout
        thread::sleep(Duration::from_secs(3));

        // Check for timeouts
        let timed_out_addrs = client_socket.check_ping_timeouts();
        assert!(timed_out_addrs.contains(&server_addr.as_ref()), "Timeout did not occur as expected");
    } 
}
