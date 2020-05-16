#[derive(Debug)]
pub struct Handshake {
    pstr: String,
    info_hash: [u8; 20],
    peer_id: [u8; 20],
}

// pub fn new_handshake(info_hash: [u8; 20], peer_id: [u8; 20]) -> Handshake {
//     Handshake {
//         pstr: "BitTorrent protocol",
//         info_hash,
//         peer_id,
//     }
// }

impl Handshake {
    fn serialize(&self) -> Vec<u8> {
        let mut buffer: Vec<u8> = Vec::new();

        // length of protocol identifier
        buffer.push(self.pstr.len() as u8);

        // the protocol identifier
        for b in self.pstr.as_bytes() {
            buffer.push(*b);
        }

        // extension bytes - all 0 in our case
        let zero: u8 = 0;
        for _ in 0..8 {
            buffer.push(zero);
        }

        // infohash
        for b in self.info_hash.iter() {
            buffer.push(*b);
        }

        // peer id
        for b in self.peer_id.iter() {
            buffer.push(*b);
        }

        buffer
    }
}
