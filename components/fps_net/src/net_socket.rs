use std::collections::HashMap;
use std::net::{UdpSocket, SocketAddr};
use std::time::{Duration, Instant};
use std::io::{Error, ErrorKind, Result};

use super::Message;

struct ReliableEntry {
    msg: Message,
    addr: SocketAddr,
    last_sent: Instant,
    attempts: u32,
}

// handle reliable
pub struct NetSocket {
    socket: UdpSocket,
    sequence: u32,
    recv_buffer: Vec<u8>,
    pub timeout: Duration,
    reliable_queue: HashMap<u32, ReliableEntry>,
}

impl NetSocket {
    /// Create a new NetSocket bound to the given address
    pub fn bind(addr: &str, timeout: Duration) -> Result<Self> {
        let socket = UdpSocket::bind(addr)?;
        socket.set_read_timeout(Some(timeout))?;
        Ok(Self {
            socket,
            sequence: 0,
            recv_buffer: vec![0u8; 65536], // max UDP size
            timeout,
            reliable_queue: HashMap::new(),
        })
    }

    /// Send a message to a given address
    pub fn send(&mut self, addr: SocketAddr, msg: &Message) -> Result<usize> {
        let bytes = msg.encode()
            .map_err(|e| Error::new(ErrorKind::Other, e.to_string()))?;
        self.socket.send_to(&bytes, addr)
    }

    pub fn send_reliable(&mut self, addr: SocketAddr, msg: &Message) -> Result<usize> {
        let sent = self.send(addr, msg)?;

        // Track reliable messages
        self.reliable_queue.insert(msg.header.sequence, ReliableEntry {
            msg: msg.clone(),
            addr,
            last_sent: Instant::now(),
            attempts: 1,
        });

        Ok(sent)
    }
    
    /// Receive a message from the socket
    pub fn recv(&mut self) -> Result<(Message, SocketAddr)> {
        match self.socket.recv_from(&mut self.recv_buffer) {
            Ok((size, src)) => {
                let msg = Message::decode(&self.recv_buffer[..size])
                    .map_err(|e| Error::new(ErrorKind::Other, e.to_string()))?;
                
                // If it's an ack, remove from reliable queue
                if msg.is_ack() {
                    if let Ok(acked_seq) = msg.decode_payload::<u32>() {
                        self.reliable_queue.remove(&acked_seq);
                    }
                }

                Ok((msg, src))
            },
            Err(e) if e.kind() == ErrorKind::WouldBlock => {
                Err(Error::new(ErrorKind::WouldBlock, "No data available"))
            },
            Err(e) => Err(e),
        }
    }

    /// Generate the next sequence number
    pub fn next_sequence(&mut self) -> u32 {
        let seq = self.sequence;
        self.sequence = self.sequence.wrapping_add(1);
        seq
    }

    pub fn resend_pending(&mut self) -> Result<()> {
        let now = Instant::now();
        for entry in self.reliable_queue.values_mut() {
            if now.duration_since(entry.last_sent) > self.timeout {
                let bytes = entry.msg.encode()
                    .map_err(|e| Error::new(ErrorKind::Other, e.to_string()))?;
                self.socket.send_to(&bytes, entry.addr)?;
                entry.last_sent = now;
                entry.attempts += 1;
            }
        }
        Ok(())
    }

    /// Set read timeout
    pub fn set_timeout(&self, timeout: Duration) -> Result<()> {
        self.socket.set_read_timeout(Some(timeout))
    }

    /// Get local socket address
    pub fn local_addr(&self) -> Result<SocketAddr> {
        self.socket.local_addr()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::thread;
    use std::time::Duration;
    use std::net::SocketAddr;

    // Test creating and binding the socket
    #[test]
    fn test_net_socket_creation() {
        let timeout = Duration::from_secs(2);
        let net_socket = NetSocket::bind("127.0.0.1:0", timeout);

        assert!(net_socket.is_ok(), "Failed to create NetSocket");

        let net_socket = net_socket.unwrap();
        assert_eq!(net_socket.timeout, timeout);
    }

    #[test]
    fn test_send_reliable_message() {
        let timeout = Duration::from_secs(2);
        let mut net_socket = NetSocket::bind("127.0.0.1:0", timeout).unwrap();

        let addr = "127.0.0.1:12345".parse::<SocketAddr>().unwrap();
        let msg = Message::new_ping(1);  // Assume Message has a `new_ping` constructor

        // Send the message
        let result = net_socket.send_reliable(addr, &msg);

        assert!(result.is_ok(), "Failed to send reliable message");

        // Check if the message was added to the reliable queue
        let reliable_entry = net_socket.reliable_queue.get(&msg.header.sequence);
        assert!(reliable_entry.is_some(), "Message not added to reliable queue");
        assert_eq!(reliable_entry.unwrap().msg, msg, "Message in queue does not match sent message");
    }

    #[test]
    fn test_receive_unreliable_message() {
        let timeout = Duration::from_secs(2);

        // Create the sender socket (sender_socket) that will send the message
        let mut sender_socket = NetSocket::bind("127.0.0.1:0", timeout).unwrap();

        // Get the dynamically assigned local address (the port assigned by the OS) for sender_socket
        let sender_local_addr = sender_socket.local_addr().unwrap();
        println!("Sender socket bound to local address: {}", sender_local_addr);

        // Create the receiver socket (receiver_socket) that will receive the message
        let mut receiver_socket = NetSocket::bind("127.0.0.1:12346", timeout).unwrap();
        println!("Receiver socket bound to local address: 127.0.0.1:12346");

        // Simulate sending a message from sender_socket to receiver_socket
        let receiver_addr = "127.0.0.1:12346".parse::<SocketAddr>().unwrap(); // receiver's address
        let msg = Message::new_ping(1); // Create a sample message (e.g., ping message)

        // Send the message from sender_socket
        let sent_size = sender_socket.send(receiver_addr, &msg).unwrap();
        println!("Sent message of size: {}", sent_size);

        // Now simulate receiving the message on receiver_socket
        let (received_msg, src) = receiver_socket.recv().unwrap();

        // Check that the received message matches the sent one
        assert_eq!(received_msg, msg, "Received message does not match the sent message");

        // Ensure that the source address is the dynamically assigned address of sender_socket
        assert_eq!(src, sender_local_addr, "Received message is from incorrect address");

        println!("Receiver socket received message from: {}", src);
    }

    #[test]
    fn test_receive_unreliable_message_with_payload() {
        let timeout = Duration::from_secs(2);

        // Create the sender socket (sender_socket) that will send the message
        let mut sender_socket = NetSocket::bind("127.0.0.1:0", timeout).unwrap();

        // Get the dynamically assigned local address (the port assigned by the OS) for sender_socket
        let sender_local_addr = sender_socket.local_addr().unwrap();
        println!("Sender socket bound to local address: {}", sender_local_addr);

        // Create the receiver socket (receiver_socket) that will receive the message
        let mut receiver_socket = NetSocket::bind("127.0.0.1:2346", timeout).unwrap();
        println!("Receiver socket bound to local address: 127.0.0.1:2346");

        // Simulate sending a message with a more complex payload (e.g., a String)
        let receiver_addr = "127.0.0.1:2346".parse::<SocketAddr>().unwrap(); // receiver's address
        let msg = Message::new_unreliable(1, &"Hello, World!"); // Create a message with a String payload

        // Send the message from sender_socket
        let sent_size = sender_socket.send(receiver_addr, &msg).unwrap();
        println!("Sent message of size: {}", sent_size);

        // Add a small delay to ensure the receiver has time to process the message
        thread::sleep(Duration::from_millis(100));

        // Now simulate receiving the message on receiver_socket
        let (received_msg, src) = receiver_socket.recv().unwrap();

        // Check that the received message matches the sent one
        assert_eq!(received_msg, msg, "Received message does not match the sent message");

        // Ensure that the source address is the dynamically assigned address of sender_socket
        assert_eq!(src, sender_local_addr, "Received message is from incorrect address");

        // Extract and check the payload of the received message
        // Assuming the payload is a String in this case
        if let Some(ref _payload) = received_msg.payload {
            // Check that the payload is of the correct type (e.g., String)
            let decoded_payload: String = received_msg.decode_payload().unwrap();
            
            // Assert that the decoded payload is correct
            assert_eq!(decoded_payload, "Hello, World!", "Decoded payload is incorrect");
        } else {
            panic!("Received message has no payload");
        }

        println!("Receiver socket received message from: {}", src);
    }

    #[test]
    fn test_resend_message() {
        let timeout = Duration::from_secs(1);
        let mut net_socket = NetSocket::bind("127.0.0.1:0", timeout).unwrap();

        let addr = "127.0.0.1:12345".parse::<SocketAddr>().unwrap();
        let msg = Message::new_ping(1);  // Assume Message has a `new_ping` constructor

        // Send the message
        net_socket.send_reliable(addr, &msg).unwrap();

        // Simulate some time passing before checking for retries
        std::thread::sleep(Duration::from_secs(2)); // Sleep longer than the timeout to trigger a resend

        // Call resend_pending() to attempt resending
        let result = net_socket.resend_pending();

        assert!(result.is_ok(), "Failed to resend pending messages");

        // Check that the message was resent
        let reliable_entry = net_socket.reliable_queue.get(&msg.header.sequence);
        assert!(reliable_entry.is_some(), "Message not found in reliable queue after retry");
        assert!(reliable_entry.unwrap().attempts > 1, "Message not retried after timeout");
    }
}