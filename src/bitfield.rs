#[derive(Debug, Eq, PartialEq)]
pub struct Bitfield {
    array: Vec<u8>,
}

impl Bitfield {
    pub fn has_piece(&self, index: usize) -> bool {
        let byte_index = index / 8;
        let offset = index % 8;
        if byte_index >= self.array.len() {
            return false;
        }

        // magic stolen from https://github.com/veggiedefender/torrent-client/blob/a83013d250dd9b4268cceace28e4cd82b07f2cbd/bitfield/bitfield.go
        return self.array[byte_index] >> (7 - offset) & 1 != 0;
    }

    pub fn set_piece(&mut self, index: usize) {
        let byte_index = index / 8;
        let offset = index % 8;

        // silently discard invalid bounded index
        if byte_index >= self.array.len() {
            return;
        }

        // magic stolen from https://github.com/veggiedefender/torrent-client/blob/a83013d250dd9b4268cceace28e4cd82b07f2cbd/bitfield/bitfield.go
        self.array[byte_index] |= 1 << (7 - offset)
    }
}

#[cfg(test)]
mod tests {
    // stolen from https://github.com/veggiedefender/torrent-client/blob/a83013d250dd9b4268cceace28e4cd82b07f2cbd/bitfield/bitfield_test.go

    #[test]
    fn has_piece_works() {
        let bf = super::Bitfield {
            array: vec![0b01010100, 0b01010100],
        };
        let outputs: Vec<bool> = vec![
            false, true, false, true, false, true, false, false, false, true, false, true, false,
            true, false, false, false, false, false, false,
        ];
        for index in 0..outputs.len() {
            assert_eq!(outputs[index], bf.has_piece(index));
        }
    }

    #[test]
    fn set_piece_works_index_4() {
        let mut bf = super::Bitfield {
            array: vec![0b01010100, 0b01010100],
            //                ^ setting this one to 1
        };
        bf.set_piece(4);

        let result_bf = super::Bitfield {
            array: vec![0b01011100, 0b01010100],
            //                ^ this one has been set to 1
        };
        assert_eq!(bf, result_bf);
    }

    #[test]
    fn set_piece_works_index_9() {
        let mut bf = super::Bitfield {
            array: vec![0b01010100, 0b01010100],
            //                         ^ already 1, no change should happen
        };
        bf.set_piece(9);

        let result_bf = super::Bitfield {
            array: vec![0b01010100, 0b01010100],
            //                         ^ still 1
        };
        assert_eq!(bf, result_bf);
    }

    #[test]
    fn set_piece_works_index_15() {
        let mut bf = super::Bitfield {
            array: vec![0b01010100, 0b01010100],
            //                               ^ setting this one to 1
        };
        bf.set_piece(15);

        let result_bf = super::Bitfield {
            array: vec![0b01010100, 0b01010101],
            //                               ^ result
        };
        assert_eq!(bf, result_bf);
    }

    #[test]
    fn set_piece_works_index_overflow() {
        let mut bf = super::Bitfield {
            array: vec![0b01010100, 0b01010100],
        };
        let result_bf = super::Bitfield {
            array: bf.array.clone(),
        };

        bf.set_piece(19); // beyond 15 is out of bounds
        assert_eq!(bf, result_bf);
    }
}
