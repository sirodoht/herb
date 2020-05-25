pub type message_id = u8;

const MSG_CHOKE: message_id = 0;
const MSG_UNCHOKE: message_id = 1;
const MSG_INTERESTED: message_id = 2;
const MSG_NOT_INTERESTED: message_id = 3;
const MSG_HAVE: message_id = 4;
const MSG_BITFIELD: message_id = 5;
const MSG_REQUEST: message_id = 6;
const MSG_PIECE: message_id = 7;
const MSG_CANCEL: message_id = 8;

// Message stores ID and payload of a message
pub struct Message {
    id: message_id,
    payload: Vec<u8>,
}

impl Message {
    // Serialize serializes a message into a buffer of the form
    // <length prefix><message ID><payload>
    // Interprets `nil` as a keep-alive message
    pub fn serialize(&self) -> Vec<u8> {
        let length: u32 = len(self.payload) + 1; // +1 for id

        let mut buf: Vec<u8> = Vec::new();

        // transforms length to big endian bytes
        let length_be: [u8; 4] = length.to_be_bytes();
        for byte in length_be {
            // add length as 4 bytes into buf
            buf.push(byte);
        }

        // 5th byte in buf is the msg id
        buf.push(self.id);

        // 6th onwards is the optional payload
        for part in self.payload {
            buf.push(part);
        }

        buf
    }
}
