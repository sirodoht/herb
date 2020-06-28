use std::str;

#[derive(Debug)]
pub struct Handshake {
    pub pstr: String,
    pub info_hash: [u8; 20],
    pub peer_id: [u8; 20],
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

pub fn read_handshake(data: &Vec<u8>) -> Handshake {
    // println!("peer handshake response");
    // println!("{:?}", data);

    // println!();
    // println!("handshake response length: {}", data.len());

    let pstr_length = data[0];
    // println!("pst length is {}", pstr_length);

    let mut pstr: [u8; 19] = [0; 19];
    for i in 0..19 {
        pstr[i] = data[i + 1]
    }
    // println!("pstr: {:?}", pstr);

    let mut extensions: [u8; 8] = [0; 8];
    for i in 0..8 {
        extensions[i] = data[i + 20];
    }
    // println!("extensions: {:?}", extensions);

    let mut info_hash: [u8; 20] = [0; 20];
    for i in 0..20 {
        info_hash[i] = data[i + 28];
    }
    // println!("info_hash: {:?}", info_hash);

    let mut peer_id: [u8; 20] = [0; 20];
    for i in 0..20 {
        peer_id[i] = data[i + 48];
    }
    // println!("peer_id: {:?}", peer_id);

    Handshake {
        pstr: str::from_utf8(&pstr).unwrap().to_owned(),
        info_hash,
        peer_id,
    }
}

#[cfg(test)]
mod tests {
    use std::str;

    use super::read_handshake;

    #[test]
    fn read_handshake_works() {
        let handshake_response: Vec<u8> = vec![
            19, 66, 105, 116, 84, 111, 114, 114, 101, 110, 116, 32, 112, 114, 111, 116, 111, 99,
            111, 108, 0, 0, 0, 0, 0, 16, 0, 5, 90, 128, 98, 192, 118, 250, 133, 232, 5, 100, 81,
            192, 217, 170, 4, 52, 154, 226, 121, 9, 45, 84, 82, 50, 57, 52, 48, 45, 98, 102, 52,
            50, 56, 107, 52, 104, 113, 107, 99, 53, 0, 0, 0, 169, 5, 255, 255, 255, 255, 255, 255,
            255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255,
            255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255,
            255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255,
            255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255,
            255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255,
            255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255,
            255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255,
            255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255,
            255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255,
            255, 255, 255, 255, 255, 255, 255, 255, 240, 0, 0, 0, 1, 1,
        ];

        let handshake = read_handshake(&handshake_response);

        assert_eq!(handshake.pstr, "BitTorrent protocol");
        assert_eq!(
            handshake.info_hash,
            [
                90, 128, 98, 192, 118, 250, 133, 232, 5, 100, 81, 192, 217, 170, 4, 52, 154, 226,
                121, 9
            ]
        );
        assert_eq!(
            str::from_utf8(&handshake.peer_id).unwrap().to_owned(),
            "-TR2940-bf428k4hqkc5"
        );
    }
}
