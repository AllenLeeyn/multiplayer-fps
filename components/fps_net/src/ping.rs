use std::collections::HashMap;
use std::net::SocketAddr;
use std::time::{Duration, Instant};

/// Represents a ping currently in-flight, tied to a specific client address
struct PingEntry {
    timestamp: Instant,
    addr: SocketAddr,
}

/// Manages ping/pong messages for multiple clients
pub struct PingManager {
    next_sequence: u32,                     // Sequence for pings
    pending_pings: HashMap<u32, PingEntry>, // Maps sequence to PingEntry
    timeout: Duration,                      // Max allowed time for a ping
}

impl PingManager {
    /// Create a new PingManager
    pub fn new(timeout: Duration) -> Self {
        Self {
            next_sequence: 0,
            pending_pings: HashMap::new(),
            timeout,
        }
    }

    /// Generate a new ping sequence for a specific address and register it
    pub fn create_ping(&mut self, addr: SocketAddr) -> u32 {
        let seq = self.next_sequence;
        self.next_sequence = self.next_sequence.wrapping_add(1); // wrap-around safe

        self.pending_pings.insert(
            seq,
            PingEntry {
                timestamp: Instant::now(),
                addr,
            },
        );

        seq
    }

    /// Handle a pong reply for a given sequence number and address
    pub fn handle_pong(&mut self, sequence: u32, addr: SocketAddr) -> Option<Duration> {
        if let Some(entry) = self.pending_pings.get_mut(&sequence) {
            if entry.addr == addr {
                let rtt = entry.timestamp.elapsed();
                self.pending_pings.remove(&sequence);
                Some(rtt)
            } else {
                None // Address mismatch, do not remove the entry
            }
        } else {
            None // No pending ping for the given sequence
        }
    }

    /// Check for timed-out pings and remove them, returns a list of addresses with timeouts
    pub fn check_timeouts(&mut self) -> Vec<SocketAddr> {
        let mut timed_out_addrs = Vec::new();

        self.pending_pings.retain(|_seq, entry| {
            if entry.timestamp.elapsed() > self.timeout {
                timed_out_addrs.push(entry.addr); // Track client address with timeout
                false
            } else {
                true
            }
        });

        timed_out_addrs
    }
}

#[cfg(test)]
mod tests {
    use super::*; // Assuming PingManager and dependencies are in the same module
    use std::net::SocketAddr;
    use std::str::FromStr;
    use std::time::Duration;

    // Helper function to create a dummy address for testing
    fn create_test_address() -> SocketAddr {
        SocketAddr::from_str("127.0.0.1:12345").unwrap()
    }

    #[test]
    fn test_create_ping() {
        let timeout = Duration::from_secs(1);
        let mut ping_manager = PingManager::new(timeout);

        let addr = create_test_address();
        let seq = ping_manager.create_ping(addr);

        // Ensure the sequence number is assigned and registered
        assert_eq!(ping_manager.pending_pings.len(), 1);
        assert!(ping_manager.pending_pings.contains_key(&seq));

        let entry = ping_manager.pending_pings.get(&seq).unwrap();
        assert_eq!(entry.addr, addr);
    }

    #[test]
    fn test_handle_pong() {
        let timeout = Duration::from_secs(1);
        let mut ping_manager = PingManager::new(timeout);

        let addr = create_test_address();
        let seq = ping_manager.create_ping(addr);

        // Simulate receiving a pong reply
        let rtt = ping_manager.handle_pong(seq, addr);

        // Ensure the ping entry is removed and RTT is calculated
        assert_eq!(ping_manager.pending_pings.len(), 0);
        assert!(rtt.is_some()); // RTT should be some value
        assert!(rtt.unwrap() > Duration::ZERO); // RTT should be positive
    }

    #[test]
    fn test_handle_pong_wrong_address() {
        let timeout = Duration::from_secs(1);
        let mut ping_manager = PingManager::new(timeout);

        let addr1 = create_test_address();
        let addr2 = SocketAddr::from_str("127.0.0.1:54321").unwrap();
        let seq = ping_manager.create_ping(addr1);

        // Simulate receiving a pong reply from a different address
        let rtt = ping_manager.handle_pong(seq, addr2);

        // Ensure no RTT is calculated, as the address doesn't match
        assert!(rtt.is_none());
        assert_eq!(ping_manager.pending_pings.len(), 1); // Ping entry should still exist
    }

    #[test]
    fn test_check_timeouts() {
        let timeout = Duration::from_secs(1);
        let mut ping_manager = PingManager::new(timeout);

        let addr1 = create_test_address();
        let addr2 = SocketAddr::from_str("127.0.0.1:54321").unwrap();

        // Create a ping for each address
        ping_manager.create_ping(addr1);
        ping_manager.create_ping(addr2);

        // Wait a bit longer than the timeout to simulate timeout for both pings
        std::thread::sleep(Duration::from_secs(2));

        let timed_out = ping_manager.check_timeouts();

        // Ensure both addresses are returned as timed-out
        assert_eq!(timed_out.len(), 2);
        assert!(timed_out.contains(&addr1));
        assert!(timed_out.contains(&addr2));

        // Ensure pings are cleared after timeout
        assert_eq!(ping_manager.pending_pings.len(), 0);
    }

    #[test]
    fn test_no_timeouts() {
        let timeout = Duration::from_secs(2);
        let mut ping_manager = PingManager::new(timeout);

        let addr = create_test_address();

        // Create a ping
        ping_manager.create_ping(addr);

        // Wait less than the timeout duration, ensuring no timeouts
        std::thread::sleep(Duration::from_secs(1));

        let timed_out = ping_manager.check_timeouts();

        // No timeouts should have occurred
        assert_eq!(timed_out.len(), 0);
        assert_eq!(ping_manager.pending_pings.len(), 1);
    }
}
