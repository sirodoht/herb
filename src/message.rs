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
    pub id: message_id,
    pub payload: Vec<u8>,
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

pub fn parse_piece(index: u32, buf: &mut Vec<u8>, msg: Message) -> u32 {
    if msg.payload.len() < 8 {
        panic!("Payload too short. {} < 8", msg.payload.len());
    }

    let parsed_index = u32::from_be_bytes([
        msg.payload[0],
        msg.payload[1],
        msg.payload[2],
        msg.payload[3],
    ]);
    if parsed_index != index {
        panic!("Expected index {}, got {}", index, parsed_index);
    }

    let begin = u32::from_be_bytes([
        msg.payload[4],
        msg.payload[5],
        msg.payload[6],
        msg.payload[7],
    ]);
    if begin as usize >= buf.len() {
        panic!("Begin offset too high. {} >= {}", begin, buf.len());
    }

    if begin + msg.payload.len() as u32 - 8 > buf.len() as u32 {
        panic!(
            "Data too long [{}] for offset {} with length {}",
            msg.payload.len() - 8,
            begin,
            buf.len()
        );
    }
    for (index, byte) in msg.payload.iter().enumerate() {
        buf[8 + index] = *byte;
    }
    return buf.len() as u32;
}

pub fn parse_have(msg: Message) -> u32 {
    if msg.payload.len() != 4 {
        panic!(
            "Expected payload length 4, got length {}",
            msg.payload.len()
        );
    }
    let index = u32::from_be_bytes([
        msg.payload[0],
        msg.payload[1],
        msg.payload[2],
        msg.payload[3],
    ]);
    return index;
}

pub fn format_have(index: i64) -> Message {
    let mut payload = vec![0u8; 4];

    let index_be = index.to_be_bytes();
    assert!(index_be.len() == 4);
    for byte in index_be.iter() {
        payload.push(*byte);
    }

    Message {
        id: MSG_HAVE,
        payload,
    }
}

pub fn format_request(index: i64, begin: i64, length: i64) -> Message {
    let mut payload = vec![0u8; 12];

    let index_be = (index as u32).to_be_bytes();
    if index_be.len() != 4 {
        println!("index_be.len() is {}", index_be.len());
    }
    for byte in index_be.iter() {
        payload.push(*byte);
    }

    let begin_be = (begin as u32).to_be_bytes();
    if begin_be.len() != 4 {
        println!("begin_be.len() is {}", begin_be.len());
    }
    for byte in begin_be.iter() {
        payload.push(*byte);
    }

    let length_be = (length as u32).to_be_bytes();
    if length_be.len() != 4 {
        println!("length_be.len() is {}", length_be.len());
    }
    for byte in length_be.iter() {
        payload.push(*byte);
    }

    Message {
        id: MSG_REQUEST,
        payload,
    }
}

pub fn new_message(data: Vec<u8>) -> Message {
    let length_u32: u32 = u32::from_be_bytes([data[0], data[1], data[2], data[3]]);
    let length: usize = length_u32.try_into().unwrap();

    let msg_id: u8 = data[4];

    let mut payload: Vec<u8> = Vec::new();
    // check if we have a payload
    if length > 1 {
        // payload starts after 4 items of length and 1 of id = 5
        // end is length+4 because `length` variable does not count itself (4 items)
        for index in 5..length + 4 {
            payload.push(data[index]);
        }
    }

    println!("NEW_MESSAGE: id: {:?}", msg_id);
    println!("NEW_MESSAGE: payload: {:?}", payload);

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
        let de = super::new_message(serialized);

        let msg = super::Message {
            id: super::MSG_CANCEL,
            payload: vec![1, 2, 3],
        };
        assert_eq!(de, msg);
    }

    #[test]
    fn message_read_no_payload() {
        let serialized: Vec<u8> = vec![0, 0, 0, 1, 2];
        let de = super::new_message(serialized);

        let msg = super::Message {
            id: super::MSG_INTERESTED,
            payload: vec![],
        };
        assert_eq!(de, msg);
    }
}
