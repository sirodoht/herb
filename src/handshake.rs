use std::str;

#[derive(Debug)]
pub struct Handshake {
    pstr: String,
    info_hash: [u8; 20],
    peer_id: [u8; 20],
}

pub fn new_handshake(info_hash: [u8; 20], peer_id: [u8; 20]) -> Handshake {
    Handshake {
        pstr: "BitTorrent protocol".to_owned(),
        info_hash,
        peer_id,
    }
}

impl Handshake {
    pub fn serialize(&self) -> Vec<u8> {
        let mut buffer: Vec<u8> = Vec::new();

        // length of protocol identifier
        buffer.push(self.pstr.len() as u8);

        // the protocol identifier
        for b in self.pstr.as_bytes() {
            buffer.push(*b);
        }

        // extension bytes - all 0 in our case
        for _ in 0..8 {
            buffer.push(0u8);
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

pub fn read_handshake(data: Vec<u8>) -> Handshake {
    println!("peer handshake response");
    println!("{:?}", data);

    println!();
    println!("handshake response length (hopefully 68): {}", data.len());

    let pstr_length = data[0];
    println!("pst length is {}", pstr_length);
    let length_in_int = u32::from_str_radix(str::from_utf8(&[pstr_length]).unwrap(), 16).unwrap();
    println!();
    println!("pstr length (hopefully 19): {}", length_in_int);

    let mut pstr: [u8; 20] = [0; 20];
    for (i, item) in (1..20).enumerate() {
        pstr[i] = item;
    }

    let mut info_hash: [u8; 20] = [0; 20];
    for (i, item) in (29..48).enumerate() {
        info_hash[i] = item;
    }

    let mut peer_id: [u8; 20] = [0; 20];
    for (i, item) in (49..68).enumerate() {
        peer_id[i] = item;
    }

    Handshake {
        pstr: str::from_utf8(&pstr).unwrap().to_owned(),
        info_hash,
        peer_id,
    }
}
