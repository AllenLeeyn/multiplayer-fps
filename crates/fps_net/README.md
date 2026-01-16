# `fps_net` - Network Communication Library

A UDP-based networking library for multiplayer game communication with reliable message delivery, connection management, and latency tracking.

## Features

- **UDP Socket Abstraction**: Low-level UDP wrapper with non-blocking I/O
- **Reliable Delivery**: Automatic retransmission for reliable messages
- **Connection Management**: Client and server socket abstractions
- **Ping/Latency**: Built-in ping/pong system for connection health
- **Message Serialization**: Binary encoding using bincode

## Quick Start

### Client

```rust
use fps_net::ClientSocket;
use std::time::Duration;

// Connect to a server
let mut client = ClientSocket::new(
    "0.0.0.0:0",           // Local bind address (0 = ephemeral port)
    "127.0.0.1:9000",      // Server address
    Duration::from_secs(5), // Ping timeout
)?;

// Send a message
let msg = fps_net::Message::new_ping(client.next_sequence());
client.send(&msg)?;

// Receive messages
if let Ok(Some(msg)) = client.recv() {
    // Handle message
}
```

### Server

```rust
use fps_net::ServerSocket;
use std::time::Duration;

// Bind to a port
let mut server = ServerSocket::bind("0.0.0.0:9000", Duration::from_secs(5))?;

// Receive from any client
if let Ok(Some((msg, client_addr))) = server.recv() {
    // Handle message from client_addr
    
    // Send response
    let response = fps_net::Message::new_pong(server.next_sequence());
    server.send(client_addr, &response)?;
}

// Broadcast to all clients
let broadcast = fps_net::Message::new_chat_message(
    server.next_sequence(),
    &fps_net::message::ChatMessagePayload {
        username: "Server".to_string(),
        text: "Hello all!".to_string(),
    },
);
server.broadcast(&broadcast)?;
```

## Message Types

The library supports various message types:

- **Control**: `Ping`, `Pong`, `ConnectRequest`, `ConnectAccept`, `ConnectDeny`, `DisconnectNotice`
- **Reliability**: `Reliable`, `Unreliable`, `Acknowledgement`
- **Game**: `JoinGame`, `GameInfo`, `ChatMessage`, `ClientList`, `StartGame`, `GameSnapShot`, `GameInput`, `GameEnd`

## Reliable Messages

Reliable messages are automatically retransmitted until acknowledged:

```rust
// Send a reliable message
client.send_reliable(&important_msg)?;

// The socket will automatically resend if no ACK is received
client.resend_pending();
```

## Ping/Latency Tracking

```rust
// Send a ping
let seq = client.send_ping()?;

// Handle pong response
if let Some(rtt) = client.handle_pong(seq) {
    println!("Round-trip time: {:?}", rtt);
}

// Check for timeouts
let timed_out = client.check_ping_timeouts();
```

## Message Serialization

Messages use bincode for efficient binary serialization:

```rust
use fps_net::{Message, message::JoinGamePayload};

let payload = JoinGamePayload {
    username: "Player1".to_string(),
};

let msg = Message::new_join_game(sequence, &payload);

// Encode to bytes
let bytes = msg.encode()?;

// Decode from bytes
let decoded = Message::decode(&bytes)?;
let payload: JoinGamePayload = decoded.decode_join_game()?;
```

## Architecture

### Modules

- **`client_socket`**: Client-side socket wrapper
- **`server_socket`**: Server-side socket with multi-client management
- **`net_socket`**: Low-level UDP socket abstraction
- **`message`**: Message types and serialization
- **`protocol`**: Protocol definitions (MessageType, MessageHeader)
- **`ping`**: Ping/pong and latency tracking
- **`util`**: Utility functions

### Connection Lifecycle

1. **Client**: Creates socket, connects to server
2. **Server**: Binds to port, accepts connections
3. **Communication**: Exchange messages (reliable or unreliable)
4. **Health**: Ping/pong maintains connection health
5. **Cleanup**: Disconnect notices and timeout handling

## Error Handling

The library uses `Box<dyn Error>` for error types. Common errors:

- Network errors (connection refused, timeout)
- Serialization errors (invalid message format)
- Protocol errors (unexpected message type)

## Thread Safety

- `ClientSocket` and `ServerSocket` are **not** thread-safe
- Use mutexes or channels for multi-threaded access
- The underlying `NetSocket` uses non-blocking I/O

## License

Part of the multiplayer-fps project.
