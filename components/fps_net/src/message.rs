use serde::{Serialize, Deserialize};
use bincode::serde::{encode_to_vec, decode_from_slice};
use bincode::config::standard;
use std::error::Error;

use super::{MessageHeader, MessageType};
use super::now_ms; 

/// Function that serializes any payload of type T into Vec<u8>
pub fn encode_payload<T: Serialize>(payload: &T) -> Result<Vec<u8>, Box<dyn Error>> {
    let config = standard();
    Ok(encode_to_vec(payload, config)?)
}

/// Generic network message
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Message {
    pub header: MessageHeader,
    pub payload: Option<Vec<u8>>,
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
                        Err(format!("Payload size mismatch: expected {} bytes, but decoded {} bytes", payload.len(), size).into())
                    }
                },
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

    pub fn new_connect_request(sequence: u32, username: &str) -> Self {
        let payload = encode_to_vec(&username, standard()).ok();
        Self::new(MessageType::ConnectRequest, sequence, payload)
    }

    pub fn new_connect_accept(sequence: u32) -> Self {
        Self::new(MessageType::ConnectAccept, sequence, None)
    }

    pub fn new_connect_deny(sequence: u32, reason: &str) -> Self {
        let payload = encode_to_vec(&reason, standard()).ok();
        Self::new(MessageType::ConnectDeny, sequence, payload)
    }

    pub fn new_disconnect_notice(sequence: u32) -> Self {
        Self::new(MessageType::DisconnectNotice, sequence, None)
    }

    pub fn new_reliable<T: Serialize>(sequence: u32, payload_obj: &T) -> Self {
        let payload = encode_to_vec(payload_obj, standard()).ok();
        Self::new(MessageType::Reliable, sequence, payload)
    }

    pub fn new_unreliable<T: Serialize>(sequence: u32, payload_obj: &T) -> Self {
        let payload = encode_to_vec(payload_obj, standard()).ok();
        Self::new(MessageType::Unreliable, sequence, payload)
    }

    pub fn new_ack(sequence: u32, acked_sequence: u32) -> Self {
        let payload = encode_to_vec(&acked_sequence, standard()).ok();
        Self::new(MessageType::Acknowledgement, sequence, payload)
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
}

#[cfg(test)]
mod tests {
    use super::{Message, MessageType};
    use std::error::Error;
    use serde::{Serialize, Deserialize};

    // Example struct to use as payload for testing
    #[derive(Debug, Serialize, Deserialize, PartialEq)]
    struct TestPayload {
        text: String,
        number: u32,
    }

    // Test encoding and decoding of the Message
    #[test]
    fn test_message_encoding_decoding() -> Result<(), Box<dyn Error>> {
        let  payload = TestPayload {
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
            invalid_field: u32,  // `u32` is incompatible with the payload which is a string
        }

        // Attempt to decode the payload into InvalidPayload
        let result: Result<InvalidPayload, _> = decoded_msg.decode_payload();

        println!("{:?}", result);

        // Assert that decoding the invalid payload fails
        assert!(result.is_err(), "Expected error while decoding invalid payload");

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
