use std::convert::{TryFrom, TryInto};

pub type message_id = u8;

pub const MSG_CHOKE: message_id = 0;
pub const MSG_UNCHOKE: message_id = 1;
pub const MSG_INTERESTED: message_id = 2;
pub const MSG_NOT_INTERESTED: message_id = 3;
pub const MSG_HAVE: message_id = 4;
pub const MSG_BITFIELD: message_id = 5;
pub const MSG_REQUEST: message_id = 6;
pub const MSG_PIECE: message_id = 7;
pub const MSG_CANCEL: message_id = 8;

// Message stores ID and payload of a message
#[derive(Debug, Eq, PartialEq)]
pub struct Message {
    id: message_id,
    payload: Vec<u8>,
}

impl Message {
    // Serialize serializes a message into a buffer of the form
    // <length prefix><message ID><payload>
    // Interprets `nil` as a keep-alive message
    pub fn serialize(&self) -> Vec<u8> {
        let length: u32 = u32::try_from(self.payload.len()).unwrap() + 1; // +1 for id

        let mut buf: Vec<u8> = Vec::new();

        // transforms length to big endian bytes
        let length_be: [u8; 4] = length.to_be_bytes();
        for byte in length_be.iter() {
            // add length as 4 bytes into buf
            buf.push(*byte);
        }

        // 5th byte in buf is the msg id
        buf.push(self.id);

        // 6th onwards is the optional payload
        for part in self.payload.iter() {
            buf.push(*part);
        }

        buf
    }
}

pub fn read_message(data: Vec<u8>) -> Message {
    let length_u32: u32 = u32::from_be_bytes([data[0], data[1], data[2], data[3]]);
    let length: usize = length_u32.try_into().unwrap();

    let msg_id: u8 = data[4];

    let mut payload: Vec<u8> = Vec::new();
    // check if we have a payload
    if length > 1 {
        // payload starts after position 5
        // and length is without counting the 4 first bytes which are the length
        for index in 5..length + 4 {
            payload.push(data[index]);
        }
    }

    Message {
        id: msg_id,
        payload,
    }
}

#[cfg(test)]
mod tests {

    #[test]
    fn message_serialize_works() {
        let msg = super::Message {
            id: super::MSG_CANCEL,
            payload: vec![1, 2, 3],
        };
        let s = msg.serialize();
        assert_eq!(s, [0, 0, 0, 4, 8, 1, 2, 3]);
    }

    #[test]
    fn message_read_works() {
        let serialized: Vec<u8> = vec![0, 0, 0, 4, 8, 1, 2, 3];
        let de = super::read_message(serialized);

        let msg = super::Message {
            id: super::MSG_CANCEL,
            payload: vec![1, 2, 3],
        };
        assert_eq!(de, msg);
    }

    #[test]
    fn message_read_no_payload() {
        let serialized: Vec<u8> = vec![0, 0, 0, 1, 2];
        let de = super::read_message(serialized);

        let msg = super::Message {
            id: super::MSG_INTERESTED,
            payload: vec![],
        };
        assert_eq!(de, msg);
    }
}
