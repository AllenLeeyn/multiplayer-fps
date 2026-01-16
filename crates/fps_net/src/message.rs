use bincode::config::standard;
use bincode::serde::{decode_from_slice, encode_to_vec};
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use std::error::Error;

use super::{MessageHeader, MessageType, now_ms};
use fps_levels::maze::Maze;

/// Function that serializes any payload of type T into Vec<u8>
pub fn encode_payload<T: Serialize>(payload: &T) -> Result<Vec<u8>, Box<dyn Error>> {
    encode_to_vec(payload, standard())
        .map_err(|e| format!("Failed to encode payload: {}", e).into())
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct JoinGamePayload {
    pub username: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct GameInfoPayload {
    pub game_name: String,
    pub maze: Maze,
    pub target_score: u32,
    pub host_username: String,
    pub state: String,
}

/// Generic network message
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Message {
    pub header: MessageHeader,
    pub payload: Option<Vec<u8>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatMessagePayload {
    pub username: String,
    pub text: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ClientListPayload {
    pub clients: Vec<String>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct PlayerSnapshot {
    pub pos: (f32, f32, f32), // x, y, angle
    pub score: u32,
    pub is_invincible: bool,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct BulletSnapshot {
    pub pos: (f32, f32, f32), // x, y, angle (angle helps with drawing tracers)
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct GameSnapShotPayload {
    pub players: HashMap<String, PlayerSnapshot>, // (id, info)
    pub bullets: Vec<BulletSnapshot>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct GameInputPayload {
    pub actions: HashSet<u8>,
    pub is_running: bool,
    pub mouse_dx: f32,
}


#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct GameEndPayload {
    pub winner: String,
}

impl Message {
    /// General constructor for control or payload messages
    pub fn new(msg_type: MessageType, sequence: u32, payload: Option<Vec<u8>>) -> Self {
        Self {
            header: MessageHeader {
                msg_type,
                sequence,
                timestamp_ms: now_ms(),
            },
            payload,
        }
    }

    /// Encode full message to bytes
    pub fn encode(&self) -> Result<Vec<u8>, Box<dyn Error>> {
        let config = standard();
        Ok(encode_to_vec(self, config)?)
    }

    /// Decode full message from bytes
    pub fn decode(bytes: &[u8]) -> Result<Self, Box<dyn Error>> {
        let config = standard();
        let (decoded, _size) = decode_from_slice::<Self, _>(bytes, config)?;
        Ok(decoded)
    }

    pub fn decode_payload<T: for<'de> Deserialize<'de>>(&self) -> Result<T, Box<dyn Error>> {
        if let Some(payload) = &self.payload {
            let config = standard();
            match decode_from_slice::<T, _>(payload, config) {
                Ok((decoded, size)) => {
                    // Check if the decoded size matches the length of the payload
                    if size == payload.len() {
                        Ok(decoded) // Successfully decoded
                    } else {
                        // If the sizes don't match, return an error
                        Err(format!(
                            "Payload size mismatch: expected {} bytes, but decoded {} bytes",
                            payload.len(),
                            size
                        )
                        .into())
                    }
                }
                Err(e) => Err(Box::new(e)),
            }
        } else {
            Err("No payload to decode".into())
        }
    }

    // --------------------------------------------------
    // Helper constructors for common message types
    // --------------------------------------------------

    pub fn new_ping(sequence: u32) -> Self {
        Self::new(MessageType::Ping, sequence, None)
    }

    pub fn new_pong(sequence: u32) -> Self {
        Self::new(MessageType::Pong, sequence, None)
    }

    pub fn new_connect_request(sequence: u32) -> Self {
        Self::new(MessageType::ConnectRequest, sequence, None)
    }

    pub fn new_connect_accept(sequence: u32) -> Self {
        Self::new(MessageType::ConnectAccept, sequence, None)
    }

    pub fn new_connect_deny(sequence: u32, reason: &str) -> Self {
        let payload = encode_to_vec(&reason, standard())
            .expect("Encoding connect_deny payload should never fail");
        Self::new(MessageType::ConnectDeny, sequence, Some(payload))
    }

    pub fn new_disconnect_notice(sequence: u32) -> Self {
        Self::new(MessageType::DisconnectNotice, sequence, None)
    }

    pub fn new_reliable<T: Serialize>(sequence: u32, payload_obj: &T) -> Self {
        let payload = encode_to_vec(&payload_obj, standard())
            .expect("Encoding reliable payload should never fail");
        Self::new(MessageType::Reliable, sequence, Some(payload))
    }

    pub fn new_unreliable<T: Serialize>(sequence: u32, payload_obj: &T) -> Self {
        let payload = encode_to_vec(&payload_obj, standard())
            .expect("Encoding unreliable payload should never fail");
        Self::new(MessageType::Unreliable, sequence, Some(payload))
    }

    pub fn new_ack(sequence: u32, acked_sequence: u32) -> Self {
        let payload = encode_to_vec(&acked_sequence, standard())
            .expect("Encoding ask payload should never fail");
        Self::new(MessageType::Acknowledgement, sequence, Some(payload))
    }

    pub fn new_join_game(sequence: u32, payload: &JoinGamePayload) -> Self {
        let payload = encode_payload(payload).expect("Encoding JoinGame payload should never fail");

        Self::new(MessageType::JoinGame, sequence, Some(payload))
    }

    pub fn new_game_info(sequence: u32, payload: &GameInfoPayload) -> Self {
        let payload = encode_payload(payload).expect("Encoding GameInfo payload should never fail");

        Self::new(MessageType::GameInfo, sequence, Some(payload))
    }

    pub fn new_chat_message(sequence: u32, payload: &ChatMessagePayload) -> Self {
        let payload_bytes =
            super::encode_payload(payload).expect("Failed to encode ChatMessage payload");
        Self::new(
            super::MessageType::ChatMessage,
            sequence,
            Some(payload_bytes),
        )
    }

    pub fn new_client_list(sequence: u32, payload: &ClientListPayload) -> Self {
        let payload_bytes =
            super::encode_payload(payload).expect("Failed to encode ClientList payload");
        Self::new(
            super::MessageType::ClientList,
            sequence,
            Some(payload_bytes),
        )
    }

    pub fn new_game_start(sequence: u32) -> Self {
        Self::new(MessageType::StartGame, sequence, None)
    }

    pub fn new_game_snapshot(sequence: u32,  payload: &GameSnapShotPayload) -> Self {
        let payload_bytes =
            super::encode_payload(payload).expect("Failed to encode GameSnapshot payload");
        Self::new(
            super::MessageType::GameSnapShot,
            sequence,
            Some(payload_bytes),
        )
    }

    pub fn new_game_input(sequence: u32,  payload: &GameInputPayload) -> Self {
        let payload_bytes =
            super::encode_payload(payload).expect("Failed to encode GameInput payload");
        Self::new(
            super::MessageType::GameInput,
            sequence,
            Some(payload_bytes),
        )
    }
    
    pub fn new_game_end(sequence: u32, winner: &str) -> Self {
        let payload = encode_to_vec(&winner, standard())
            .expect("Encoding game_end payload should never fail");
        Self::new(MessageType::GameEnd, sequence, Some(payload))
    }

    // --------------------------------------------------
    // Helper methods to check message type
    // --------------------------------------------------

    pub fn is_ping(&self) -> bool {
        self.header.msg_type == MessageType::Ping
    }

    pub fn is_pong(&self) -> bool {
        self.header.msg_type == MessageType::Pong
    }

    pub fn is_reliable(&self) -> bool {
        self.header.msg_type == MessageType::Reliable
    }

    pub fn is_unreliable(&self) -> bool {
        self.header.msg_type == MessageType::Unreliable
    }

    pub fn is_ack(&self) -> bool {
        self.header.msg_type == MessageType::Acknowledgement
    }

    pub fn is_connect_request(&self) -> bool {
        self.header.msg_type == MessageType::ConnectRequest
    }

    pub fn is_connect_accept(&self) -> bool {
        self.header.msg_type == MessageType::ConnectAccept
    }

    pub fn is_connect_deny(&self) -> bool {
        self.header.msg_type == MessageType::ConnectDeny
    }

    pub fn is_disconnect_notice(&self) -> bool {
        self.header.msg_type == MessageType::DisconnectNotice
    }

    pub fn is_join_game(&self) -> bool {
        self.header.msg_type == MessageType::JoinGame
    }

    pub fn is_game_info(&self) -> bool {
        self.header.msg_type == MessageType::GameInfo
    }

    pub fn is_chat_message(&self) -> bool {
        self.header.msg_type == MessageType::ChatMessage
    }

    pub fn is_client_list(&self) -> bool {
        self.header.msg_type == MessageType::ClientList
    }

    pub fn is_game_start(&self) -> bool {
        self.header.msg_type == MessageType::StartGame
    }

    pub fn is_game_snapshot(&self) -> bool {
        self.header.msg_type == MessageType::GameSnapShot
    }

    pub fn is_game_input(&self) -> bool {
        self.header.msg_type == MessageType::GameInput
    }

    pub fn is_game_end(&self) -> bool {
        self.header.msg_type == MessageType::GameEnd
    }

    pub fn decode_connect_deny(&self) -> Result<String, Box<dyn std::error::Error>> {
        if !self.is_connect_deny() {
            return Err("Message is not ConnectDeny".into());
        }
        self.decode_payload()
    }

    pub fn decode_join_game(&self) -> Result<JoinGamePayload, Box<dyn std::error::Error>> {
        if !self.is_join_game() {
            return Err("Message is not JoinGame".into());
        }
        self.decode_payload()
    }

    pub fn decode_game_info(&self) -> Result<GameInfoPayload, Box<dyn std::error::Error>> {
        if !self.is_game_info() {
            return Err("Message is not GameInfo".into());
        }
        self.decode_payload()
    }

    pub fn decode_chat_message(&self) -> Result<ChatMessagePayload, Box<dyn std::error::Error>> {
        if !self.is_chat_message() {
            return Err("Message is not ChatMessage".into());
        }
        self.decode_payload()
    }

    pub fn decode_client_list(&self) -> Result<ClientListPayload, Box<dyn std::error::Error>> {
        if !self.is_client_list() {
            return Err("Message is not ClientList".into());
        }
        self.decode_payload()
    }

    pub fn decode_game_snapshot(&self) -> Result<GameSnapShotPayload, Box<dyn std::error::Error>> {
        if !self.is_game_snapshot() {
            return Err("Message is not GameSnapShot".into());
        }
        self.decode_payload()
    }
    
    pub fn decode_game_input(&self) -> Result<GameInputPayload, Box<dyn std::error::Error>> {
        if !self.is_game_input() {
            return Err("Message is not GameInput".into());
        }
        self.decode_payload()
    }

    pub fn decode_game_end(&self) -> Result<GameEndPayload, Box<dyn std::error::Error>> {
        if !self.is_game_end() {
            return Err("Message is not GameEnd".into());
        }
        self.decode_payload()
    }
}

#[cfg(test)]
mod tests {
    use super::{Message, MessageType};
    use serde::{Deserialize, Serialize};
    use std::error::Error;

    // Example struct to use as payload for testing
    #[derive(Debug, Serialize, Deserialize, PartialEq)]
    struct TestPayload {
        text: String,
        number: u32,
    }

    // Test encoding and decoding of the Message
    #[test]
    fn test_message_encoding_decoding() -> Result<(), Box<dyn Error>> {
        let payload = TestPayload {
            text: "Hello, world!".to_string(),
            number: 42,
        };

        // Create a message with a payload
        let msg = Message::new_reliable(1, &payload);

        // Encode the message into bytes
        let encoded_msg = msg.encode()?;

        // Decode the message back from bytes
        let decoded_msg = Message::decode(&encoded_msg)?;

        // Ensure that the header matches and the payload is decoded correctly
        assert_eq!(msg.header.msg_type, decoded_msg.header.msg_type);
        assert_eq!(msg.header.sequence, decoded_msg.header.sequence);
        assert_eq!(msg.header.timestamp_ms, decoded_msg.header.timestamp_ms);

        // Now decode the payload
        let decoded_payload: TestPayload = decoded_msg.decode_payload()?;
        assert_eq!(decoded_payload.text, "Hello, world!");
        assert_eq!(decoded_payload.number, 42);

        Ok(())
    }

    // Test decoding with invalid payload
    #[test]
    fn test_valid_payload_decoding() -> Result<(), Box<dyn std::error::Error>> {
        // Create a valid payload of type TestPayload
        let payload = TestPayload {
            number: 123,
            text: "test".to_string(),
        };

        // Create a reliable message with the encoded payload
        let msg = Message::new_reliable(1, &payload);

        // Encode the message into bytes
        let encoded_msg = msg.encode()?;

        // Decode the message back from bytes
        let decoded_msg = Message::decode(&encoded_msg)?;

        // Now decode the payload
        let decoded_payload: TestPayload = decoded_msg.decode_payload()?;

        // Assert that the payload is decoded correctly
        assert_eq!(decoded_payload.text, "test");
        assert_eq!(decoded_payload.number, 123);

        Ok(())
    }

    #[test]
    fn test_invalid_payload_decoding() -> Result<(), Box<dyn std::error::Error>> {
        // Create a valid payload of type TestPayload
        let payload = TestPayload {
            number: 123,
            text: "test".to_string(),
        };

        // Create a reliable message with the encoded payload
        let msg = Message::new_reliable(1, &payload);

        // Encode the message into bytes
        let encoded_msg = msg.encode()?;

        // Decode the message back from bytes
        let decoded_msg = Message::decode(&encoded_msg)?;

        // Try to decode the payload as a completely different struct
        #[derive(Serialize, Deserialize, Debug)]
        struct InvalidPayload {
            invalid_field: u32, // `u32` is incompatible with the payload which is a string
        }

        // Attempt to decode the payload into InvalidPayload
        let result: Result<InvalidPayload, _> = decoded_msg.decode_payload();

        println!("{:?}", result);

        // Assert that decoding the invalid payload fails
        assert!(
            result.is_err(),
            "Expected error while decoding invalid payload"
        );

        // Optionally, check the error type to ensure it's a deserialization error
        if let Err(e) = result {
            println!("Expected error: {:?}", e);
        }

        Ok(())
    }

    // Test encoding and decoding a message without a payload
    #[test]
    fn test_message_without_payload() -> Result<(), Box<dyn Error>> {
        let msg = Message::new(MessageType::Reliable, 1, None);

        // Encode the message into bytes
        let encoded_msg = msg.encode()?;

        // Decode the message back from bytes
        let decoded_msg = Message::decode(&encoded_msg)?;

        // Ensure that the header matches and there is no payload
        assert_eq!(msg.header.msg_type, decoded_msg.header.msg_type);
        assert_eq!(msg.header.sequence, decoded_msg.header.sequence);
        assert!(decoded_msg.payload.is_none());

        Ok(())
    }
}
