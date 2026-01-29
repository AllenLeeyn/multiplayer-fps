# `fps_net` - Network Communication Library

A UDP-based networking library for multiplayer game communication with reliable message delivery, connection management, and latency tracking. In this document: [Quick Start](#quick-start), [Protocol and message format](#protocol-and-message-format), [Message types](#message-types), [Reliable messages](#reliable-messages), [NetSocket](#netsocket-low-level-setup), [ClientSocket](#clientsocket-setup), [ServerSocket](#serversocket-setup), [Ping and PingManager](#ping-and-pong), [Serialization](#message-serialization-wire-format), [Architecture](#architecture).

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

## Protocol and Message Format

### Message header (`protocol.rs`)

The protocol is defined in `protocol.rs`. Every message on the wire carries a **header** so the receiver can route, order, and time it without parsing the body.

**`MessageHeader`** contains:

| Field          | Type     | Purpose |
|----------------|----------|---------|
| `msg_type`     | `MessageType` | Identifies the kind of message (e.g. Ping, JoinGame, GameSnapShot). |
| `sequence`     | `u32`    | Ordering and deduplication; used for reliable delivery and ACKs. |
| `timestamp_ms` | `u64`    | When the message was created (e.g. for latency or ordering). |

**Why a header:** The receiver can dispatch (e.g. “is this a ping or game snapshot?”), match ACKs to sequences, and detect duplicates or timeouts without decoding the payload.

### Message shape

**`Message`** is a single struct used for all network messages:

```rust
pub struct Message {
    pub header: MessageHeader,
    pub payload: Option<Vec<u8>>,
}
```

- **`header`**: Always present; carries `msg_type`, `sequence`, `timestamp_ms`.
- **`payload`**: Optional **opaque bytes**. For message types that carry data, the payload is the bincode-encoded bytes of a specific Rust type (e.g. `JoinGamePayload`, `GameSnapShotPayload`).

### Why compact binary encoding matters (and the crate we use)

Multiplayer games send many small messages (inputs, snapshots, chat) at high frequency. Keeping each message **as small as possible** is important:

- **Bandwidth**: Smaller packets use less network capacity. That matters on crowded Wi‑Fi, mobile, or when many players are in one game.
- **UDP and MTU**: UDP datagrams are often limited by the network MTU (e.g. 1500 bytes). Compact encoding lets more logical messages fit in one datagram and reduces fragmentation.
- **Latency**: Less data to send and receive means less time on the wire and in the kernel, which helps keep latency and jitter low.
- **Predictability**: Fixed-size or small variable-size binary layouts are easier to reason about than large text payloads when tuning for throughput and packet rate.

**Bit packing** in the broad sense means representing data in a **compact binary form** instead of a verbose text format (e.g. JSON). This crate does not pack individual bits by hand; it uses **compact binary serialization** so that integers, enums, and structs are encoded in a small, well-defined binary format rather than as human-readable text.

We use the **[bincode](https://docs.rs/bincode)** crate to achieve this. Bincode:

- Serializes Rust types to a **binary** format (not text). For example, a `u32` is 4 bytes; enums and structs have no extra field names or delimiters.
- Is **compact** compared to JSON or XML: no key names, no whitespace, and efficient representation of numbers and collections.
- Works with **Serde**: any `Serialize`/`Deserialize` type can be encoded as message payloads with minimal code.
- Produces a **deterministic** byte stream for the same value, which helps with debugging and optional checksums.

All `Message` encoding and decoding (header and payload bytes) goes through bincode, so every message on the wire is in this compact binary form.

**Encoding/decoding:**

1. **Whole message**: `Message::encode()` and `Message::decode()` serialize/deserialize the entire `Message` (header + optional payload bytes) with bincode. The payload stays as raw bytes in the wire format.
2. **Payload by type**: The **meaning** of the payload depends on `header.msg_type`. The crate provides:
   - **Typed constructors** (e.g. `Message::new_join_game(seq, &payload)`) that encode the correct Rust type into `payload` and set `msg_type`.
   - **Typed decoders** (e.g. `msg.decode_join_game()`) that check `msg_type` and decode `payload` into the corresponding type (e.g. `JoinGamePayload`).  
   So you always encode/decode the payload according to the message type; the header tells you which type to use.

## Message types

Each variant of `MessageType` corresponds to a fixed payload (or none). Use the matching `new_*` to send and `decode_*` / `is_*` to receive.

| MessageType        | Payload | Description |
|--------------------|--------|-------------|
| **Ping**           | None   | Latency probe. |
| **Pong**           | None   | Latency reply. |
| **ConnectRequest** | None   | Client asks to join. |
| **ConnectAccept**  | None   | Server accepts. |
| **ConnectDeny**    | `String` | Server rejects (reason). |
| **DisconnectNotice** | None | Peer is disconnecting. |
| **Reliable**       | Any `T: Serialize` | Reliable application payload. |
| **Unreliable**     | Any `T: Serialize` | Unreliable application payload. |
| **Acknowledgement**| `u32` (acked sequence) | ACK for a reliable message. |
| **JoinGame**       | `JoinGamePayload` (username) | Client joins with a name. |
| **GameInfo**       | `GameInfoPayload` (game name, maze, target score, host, state) | Server sends lobby/game info. |
| **ChatMessage**    | `ChatMessagePayload` (username, text) | Chat line. |
| **ClientList**     | `ClientListPayload` (list of client identifiers, e.g. usernames) | Server sends list of players. |
| **StartGame**      | None   | Server signals game start. |
| **GameSnapShot**   | `GameSnapShotPayload` (players, bullets) | Server sends game state. |
| **GameInput**      | `GameInputPayload` (actions, is_running, mouse_dx) | Client sends input. |
| **GameEnd**        | `GameEndPayload` (winner) | Game over, winner ID. |

For **Reliable** and **Unreliable**, the payload type is application-defined; the receiver must call `decode_payload::<T>()` with the expected type. All other rows use the fixed payload type and matching `new_*` / `decode_*` helpers.

## Reliable Messages

Reliable messages are automatically retransmitted until acknowledged:

```rust
// Send a reliable message
client.send_reliable(&important_msg)?;

// Call resend_pending() periodically (e.g. in the game loop) to retransmit until ACK
client.resend_pending();
```

---

## NetSocket (low-level setup)

`NetSocket` is the low-level UDP wrapper used by `ClientSocket` and `ServerSocket`. It handles binding, non-blocking I/O, message encode/decode, and the reliable send queue.

### Struct setup

`NetSocket` holds:

| Field             | Type                    | Purpose |
|-------------------|-------------------------|---------|
| `socket`          | `UdpSocket`             | OS UDP socket. |
| `sequence`        | `u32`                   | Next sequence number for outgoing messages (`next_sequence()`). |
| `recv_buffer`     | `Vec<u8>`               | Buffer for `recv_from` (max UDP size, 65536 bytes). |
| `timeout`         | `Duration` (pub)        | Used for reliable retransmit and (optionally) read timeout. |
| `reliable_queue`  | `HashMap<u32, ReliableEntry>` | Pending reliable messages: sequence → message, address, last send time, attempt count. |
| `local_ip`        | `String` (pub)         | Detected local/LAN IP (for display). |
| `public_ip`       | `String` (pub)         | Detected public IP (e.g. via ipify). |

`ReliableEntry` (internal) stores the message, destination address, `Instant` of last send, and number of send attempts.

### Non-blocking nature

The underlying `UdpSocket` is set to **non-blocking** in `bind()` via `socket.set_nonblocking(true)`. So:

- **`recv()`**: If no datagram is available, it returns immediately with an error (e.g. `WouldBlock`). The game loop can poll without blocking and do other work (e.g. simulation, ping, resend).
- **`send()` / `send_to`**: Sends are typically non-blocking as well for UDP; the kernel may return “would block” only under heavy load.

The application is responsible for handling `WouldBlock` (e.g. with `util::ignore_would_block` or by looping with a short sleep).

### bind()

**`NetSocket::bind(addr, timeout)`**:

1. Creates a `UdpSocket` and binds it to `addr` (e.g. `"0.0.0.0:9000"` or `"127.0.0.1:0"` for an ephemeral port).
2. Sets the socket to **non-blocking**.
3. Optionally detects **local IP** (e.g. via a temporary socket to an external address) and **public IP** (e.g. via `https://api.ipify.org`); falls back to `127.0.0.1` if detection fails.
4. Allocates the **recv buffer** (65536 bytes) and initializes **sequence** to 0 and **reliable_queue** to empty.
5. Stores the given **timeout** for use by the reliable queue and optional read timeout.

You only create a `NetSocket` by calling `bind()`; there is no separate “connect” step (UDP is connectionless). `ClientSocket` and `ServerSocket` wrap a `NetSocket` and add a fixed server address (client) or a map of clients (server).

### Reliable setup

- **Sending**: `send_reliable(addr, msg)` encodes the message, sends it with `send_to`, and **inserts** an entry into `reliable_queue` keyed by `msg.header.sequence` (message, address, `Instant::now()`, attempts = 1).
- **ACK handling**: When **`recv()`** returns a message, if the message type is **Acknowledgement**, the socket decodes the payload as the acked sequence number and **removes** that sequence from `reliable_queue`. So once the peer sends an ACK, the message is no longer retransmitted.
- **Retransmit**: The application must call **`resend_pending()`** periodically. It walks `reliable_queue` and, for any entry whose `last_sent` is older than `timeout`, re-sends the message and updates `last_sent` and `attempts`. Entries that still fail to send are returned as `(sequence, addr)` for the app to handle. Reliable delivery is “at least once” until an ACK is received; the peer must deduplicate by sequence if needed.

So the “reliable setup” is: use `send_reliable` for messages that must be acknowledged, run `resend_pending()` in the loop, and implement the protocol so the receiver sends ACKs for those messages. `NetSocket` does not send ACKs itself; the higher layer (e.g. game server) does that when it receives a reliable message.

### recv() and send()

- **`recv(&mut self) -> Result<(Message, SocketAddr), Box<dyn Error>>`**  
  Reads one datagram from the socket into the internal buffer, decodes it as a `Message`, and returns the message and source address. If the decoded message is an **Acknowledgement**, the socket also removes the corresponding sequence from `reliable_queue`. On no data (non-blocking), the underlying `recv_from` returns an error (e.g. `WouldBlock`), which is propagated.

- **`send(&mut self, addr: SocketAddr, msg: &Message) -> Result<usize, Box<dyn Error>>`**  
  Encodes the message with bincode and sends the bytes to `addr` via `send_to`. Returns the number of bytes sent. This is **unreliable** from the crate’s point of view: no queue, no retransmit. For reliable delivery, use `send_reliable()` and `resend_pending()` as above.

---

## ClientSocket (setup)

`ClientSocket` is the client-side API: one fixed server address, one `NetSocket`, and a `PingManager` for heartbeat and RTT.

### Struct setup

| Field           | Type         | Purpose |
|-----------------|--------------|---------|
| `socket`        | `NetSocket`  | Low-level UDP socket (non-blocking, bind, reliable queue). |
| `server_addr`   | `SocketAddr` | The only peer: all sends go here; recv accepts only messages from this address. |
| `ping_manager`  | `PingManager` | Tracks in-flight pings to the server and timeouts. |

There is no separate “connect” call. The client is “connected” to `server_addr` from construction; UDP is connectionless, so the socket is just bound locally and the server address is stored for filtering and sending.

### Constructor: `ClientSocket::new(local_addr, server_addr, ping_timeout)`

1. **Bind**: Calls `NetSocket::bind(local_addr, ping_timeout)` so the OS binds the UDP socket to `local_addr` (e.g. `"0.0.0.0:0"` for any interface and an ephemeral port). The socket is non-blocking and has the given timeout for the reliable queue.
2. **Server address**: Parses `server_addr` (e.g. `"127.0.0.1:9000"`) into a `SocketAddr`. Returns an error if the string is invalid.
3. **Ping manager**: Creates `PingManager::new(ping_timeout)` with the same `Duration`, so ping timeouts and the socket’s reliable timeout are aligned.

Returns `Ok(ClientSocket { socket, server_addr, ping_manager })` or an error (bind or parse failure).

### Receive: only from the server

**`recv(&mut self) -> Result<Option<Message>, Box<dyn Error>>`**  
Calls `socket.recv()`. If a message is received:
- **From `server_addr`**: Returns `Ok(Some(msg))`.
- **From any other address**: Returns `Ok(None)` and drops the message (so the client ignores stray or spoofed packets).

On recv error (e.g. non-blocking with no data), the error is returned. The app typically handles `WouldBlock` by looping or yielding.

### Send: always to the server

- **`send(&mut self, msg: &Message)`**  
  Sends the message to `server_addr` via `socket.send(server_addr, msg)` (unreliable).
- **`send_reliable(&mut self, msg: &Message)`**  
  Sends to `server_addr` and enqueues for retransmit via `socket.send_reliable(server_addr, msg)`.
- **`resend_pending(&mut self)`**  
  Delegates to `socket.resend_pending()`; returned `(sequence, addr)` will use `server_addr` for a client.

Sequence numbers are provided by **`next_sequence(&mut self)`**, which forwards to `socket.next_sequence()`.

### Ping / pong (heartbeat and RTT)

- **`send_ping()`**: Creates a ping for `server_addr` in the ping manager, builds `Message::new_ping(seq)`, sends it to the server, and returns the sequence.
- **`handle_pong(seq)`**: When the client receives a pong from the server, call this with the pong’s sequence; it returns `Some(rtt)` if that ping was pending, and clears the pending ping.
- **`handle_ping(seq)`**: Provided for symmetry; the **server** answers pings with pongs. The client uses this only if it ever had to respond to a server ping (e.g. in some designs the server pings the client). It returns a `Message::new_pong(seq)` to send back.
- **`check_ping_timeouts()`**: Delegates to the ping manager; returns a list of addresses that failed to pong in time. For a client there is at most one address (the server); if non-empty, the app can treat the server as unreachable or disconnected.

### Other

- **`local_addr()`**: Returns the socket’s local bind address (e.g. for display or NAT).

---

## ServerSocket (setup)

`ServerSocket` is the server-side API: one `NetSocket` bound to a port, a map of **clients** (address → last-seen), and a `PingManager` for multiple peers. The server receives from any client and sends to specific addresses or broadcasts to all known clients.

### Struct setup

| Field            | Type                              | Purpose |
|------------------|-----------------------------------|---------|
| `socket`         | `NetSocket`                       | Low-level UDP socket (non-blocking, bind, reliable queue). |
| `clients`        | `HashMap<SocketAddr, ClientInfo>` | Known clients: each address has a `ClientInfo` with `last_seen` (timestamp in ms). |
| `ping_manager`   | `PingManager`                     | Tracks in-flight pings to multiple client addresses. |
| `timeout`        | `Duration`                        | Used for reliable retransmit, ping timeout, and stale-client removal. |

**`ClientInfo`** (public) has one field: **`last_seen: u64`** (milliseconds since epoch). Any time the server receives a message from an address, it updates that address’s `last_seen` so the server can later drop clients that have been idle longer than `timeout`.

### Constructor: `ServerSocket::bind(local_addr, timeout)`

1. **Bind**: Calls `NetSocket::bind(local_addr, timeout)` so the UDP socket is bound to `local_addr` (e.g. `"0.0.0.0:9000"`). The socket is non-blocking and uses `timeout` for the reliable queue.
2. **Clients**: Initializes `clients` as an empty `HashMap`. Clients are added implicitly when they first send a message (see **Receive**).
3. **Ping manager**: Creates `PingManager::new(timeout)` so ping timeouts match the server’s timeout.
4. **Timeout**: Stores `timeout` for use by `remove_stale_clients()` and the reliable/ping logic.

Returns `Ok(ServerSocket { ... })` or an error on bind failure.

### Receive: from any client, and marking clients alive

**`recv(&mut self) -> Result<Option<(Message, SocketAddr)>, Box<dyn Error>>`**  
Calls `socket.recv()`. If a datagram is received:
- Decodes it as a `Message` and gets the source `addr`.
- **Marks the client alive**: `mark_client_alive(addr)` inserts or updates `clients[addr]` with `last_seen = now_ms()`. So every received message (from any address) adds or refreshes that address in the client set.
- Returns `Ok(Some((msg, addr)))`.

On recv error (e.g. non-blocking with no data), the error is returned. The server does not filter by address; it accepts messages from any sender and uses the client map for broadcast and staleness.

### Send: to one client or broadcast

- **`send(addr, msg)`**: Sends the message to `addr` via `socket.send(addr, msg)` (unreliable).
- **`send_reliable(addr, msg)`**: Sends to `addr` and enqueues for retransmit via `socket.send_reliable(addr, msg)`.
- **`resend_pending()`**: Delegates to `socket.resend_pending()`; can involve multiple client addresses.
- **`broadcast(msg)`**: Sends `msg` to every address currently in `clients`. Returns a list of addresses that failed to send (e.g. for logging or removal).
- **`broadcast_reliable(msg)`**: Same as broadcast but uses `send_reliable` for each client; each send enqueues that message for that address.

**`next_sequence(&mut self)`** forwards to `socket.next_sequence()` for sequence numbers.

### Ping / pong (heartbeat and RTT)

- **`send_ping(addr)`**: Creates a ping for `addr` in the ping manager, builds `Message::new_ping(seq)`, sends it to that client, and returns the sequence.
- **`handle_pong(addr, seq)`**: Call when a pong is received from `addr` with the given sequence. Marks the client alive and delegates to the ping manager; returns `Some(rtt)` if that ping was pending.
- **`handle_ping(addr, seq)`**: Call when a **ping** is received from a client (e.g. client is measuring RTT to the server). Marks the client alive and returns `(addr, Message::new_pong(seq))` so the server can send the pong back to `addr`.
- **`check_ping_timeouts()`**: Delegates to the ping manager; returns the list of addresses that failed to pong in time. The app can remove those from the game or client list.

### Client list and stale removal

- **`client_list()`**: Returns a copy of all addresses currently in `clients` (every address that has sent at least one message and not yet been removed as stale).
- **`remove_stale_clients()`**: Removes from `clients` any address whose `last_seen` is older than `timeout` (compared using current time in ms). Returns the list of removed addresses. Call this periodically so the server drops idle or dead clients and does not broadcast to them.

### Other

- **`local_addr()`**: Returns the socket’s local bind address (`Result<SocketAddr, ...>`).
- **`local_ip()`**: Returns the detected local/LAN IP string from `NetSocket` (`Result<String, ...>`).
- **`public_addr()`**: Returns `Ok("<local_ip>:<port>")` so the host can share a single address (e.g. for port-forwarding or display); errors if `local_addr()` or `local_ip()` fails.

---

## Ping and Pong

**Ping** and **Pong** are small, payload-free messages used to measure round-trip time (RTT) and check that the peer is still responsive.

- **Ping**: The sender allocates a sequence number, sends a `Message` with `MessageType::Ping` and that sequence, and records the send time and peer address.
- **Pong**: The receiver gets the ping and sends back a `Message` with `MessageType::Pong` and the **same** sequence number. No payload; the sequence links the reply to the original ping.
- **RTT**: When the sender receives the pong, it looks up the sequence, computes elapsed time since the ping was sent, and returns that as the round-trip time. That gives you latency to the peer.

Typical use: the client (or server) periodically sends pings, and when pongs arrive it can display latency or treat missing pongs as a sign of a bad or disconnected peer.

---

## PingManager: setup and what it helps with

Each `ClientSocket` and `ServerSocket` owns a **`PingManager`** that tracks in-flight pings and timeouts.

### Setup

- **Client**: `ClientSocket::new(local_addr, server_addr, ping_timeout)` creates a `PingManager::new(ping_timeout)` internally. The same `Duration` is used as the socket’s timeout and as the max time to wait for a pong before considering the ping timed out.
- **Server**: `ServerSocket::bind(addr, timeout)` creates a `PingManager::new(timeout)`. The server can ping many clients; the manager stores one pending ping per sequence and associates each with a client address.

So you don’t construct `PingManager` yourself; it’s created with the socket and the timeout you pass in.

### What the PingManager does

- **`create_ping(addr)`**: Generates the next sequence number, records a pending ping (timestamp + address), and returns the sequence. The caller sends a `Message::new_ping(seq)` to that address.
- **`handle_pong(seq, addr)`**: If there is a pending ping for that sequence and address, computes RTT (now − timestamp), removes the pending ping, and returns `Some(rtt)`. If the address doesn’t match or the sequence is unknown, returns `None` (and does not remove other entries).
- **`check_timeouts()`**: Removes any pending ping older than the configured timeout and returns the list of addresses that timed out. The application can use that list to disconnect stale clients or show “connection lost.”

### What it helps with

- **Latency (RTT)**: You get a concrete duration for each successful ping–pong, so you can show ping in the UI or tune game logic.
- **Connection health**: If pongs stop arriving, `check_timeouts()` tells you which addresses failed to respond in time, so you can treat them as disconnected or unreachable.
- **Stale cleanup**: The manager only keeps pending pings for the timeout period; old entries are removed by `check_timeouts()`, so the in-memory state doesn’t grow forever.

### Example (client)

```rust
// Send a ping
let seq = client.send_ping()?;

// When you receive a message, if it's a pong, handle it
if msg.is_pong() {
    if let Some(rtt) = client.handle_pong(msg.header.sequence) {
        println!("Round-trip time: {:?}", rtt);
    }
}

// Periodically check for timeouts (e.g. no pong within ping_timeout)
let timed_out = client.check_ping_timeouts();
if !timed_out.is_empty() {
    // Server didn't respond in time
}
```

## Message serialization (wire format)

The whole `Message` (header + optional payload bytes) is serialized with **bincode**. Payload bytes are produced by typed constructors and consumed by typed decoders; see [Protocol and message format](#protocol-and-message-format) and [Message types](#message-types).

```rust
use fps_net::{Message, message::JoinGamePayload};

// Build message with typed payload (payload is encoded internally)
let payload = JoinGamePayload { username: "Player1".to_string() };
let msg = Message::new_join_game(sequence, &payload);

// Wire: encode full message to bytes
let bytes = msg.encode()?;

// Wire: decode full message from bytes
let decoded = Message::decode(&bytes)?;
// Then decode payload by message type
let payload: JoinGamePayload = decoded.decode_join_game()?;
```

## Architecture

### Modules

- **`protocol`**: Defines `MessageType` and `MessageHeader` (what goes on every message and why).
- **`message`**: Defines `Message` (header + optional payload bytes), payload types per `MessageType`, and encode/decode (full message + typed payload).
- **`client_socket`**: Client-side socket wrapper (single server address).
- **`server_socket`**: Server-side socket with multi-client management.
- **`net_socket`**: Low-level UDP socket and reliable send queue.
- **`ping`**: Ping/pong and latency tracking (`PingManager`).
- **`util`**: Helpers (e.g. `now_ms()`, `ignore_would_block`).

### Dependencies

- **bincode** (with serde): Binary serialization for all messages.
- **serde**: Serialize/Deserialize for headers and payloads.
- **fps_levels**: Used by `GameInfoPayload` (maze data); re-exported in game crate.
- **reqwest** (blocking): Used in `NetSocket::bind()` to detect public IP via `https://api.ipify.org`; falls back to `127.0.0.1` if the request fails (e.g. offline).

### Connection Lifecycle

Because UDP is connectionless, there is no TCP-style `connect()` or `accept()`; the server learns about clients when they first send a message, and the client simply targets a server address.

1. **Client**: Creates socket and binds locally; targets a server address (all sends go there).
2. **Server**: Binds to a port; clients are added to the client map when they first send a message.
3. **Communication**: Exchange messages (reliable or unreliable).
4. **Health**: Ping/pong used as a heartbeat to detect idle or dead peers.
5. **Cleanup**: Disconnect notices, ping timeouts, and `remove_stale_clients()` (server) or `check_ping_timeouts()` (client).

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
