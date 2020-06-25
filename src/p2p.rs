use crossbeam::channel::{Receiver, Sender};
use sha1::{Digest, Sha1};
use std::convert::TryInto;
use std::net::IpAddr;
use std::time::Duration;

use crate::client;
use crate::message;
use crate::p2p;

pub static PEER_ID_STRING: &str = "kjh29409k8hj0wgej6c1";

#[derive(Debug)]
pub struct Peer {
    pub ip: IpAddr,
    pub port: u16,
}

#[derive(Copy, Clone)]
pub struct PieceWork {
    pub index: i64,
    pub hash: [u8; 20],
    pub length: i64,
}

pub struct PieceResult {
    pub index: i64,
    pub buf: Vec<u8>,
}

pub struct PieceProgress<'a> {
    pub index: i64,
    pub client: &'a mut client::Client,
    pub buf: Vec<u8>,
    pub downloaded: i64,
    pub requested: i64,
    pub backlog: i64,
}

impl PieceProgress<'_> {
    pub fn read_message(&mut self) {
        let msg = self.client.read().unwrap();

        if msg.id == message::MSG_UNCHOKE {
            self.client.choked = false;
        } else if msg.id == message::MSG_CHOKE {
            self.client.choked = true;
        } else if msg.id == message::MSG_HAVE {
            let index = message::parse_have(msg);
            self.client.bitfield.set_piece(index as i64);
        } else if msg.id == message::MSG_PIECE {
            let n = message::parse_piece(self.index as u32, &mut self.buf, msg);
            self.downloaded += n as i64;
            self.backlog -= 1;
        }
    }
}

pub fn attempt_download_piece(c: &mut client::Client, pw: PieceWork) -> Vec<u8> {
    let mut state = PieceProgress {
        index: pw.index,
        client: c,
        buf: vec![0u8; pw.length as usize],
        downloaded: 0,
        requested: 0,
        backlog: 0,
    };

    state
        .client
        .conn
        .set_read_timeout(Some(Duration::new(30, 0)))
        .unwrap();

    while state.downloaded < pw.length {
        // If unchoked, send requests until we have enough unfulfilled requests
        if !state.client.choked {
            let max_block_size = 16384;

            while state.backlog < max_block_size && state.requested < pw.length {
                let mut block_size = max_block_size;

                // Last block might be shorter than the typical block
                if pw.length - state.requested < block_size {
                    block_size = pw.length - state.requested;
                }

                state
                    .client
                    .send_request(pw.index, state.requested, block_size);
                state.backlog += 1;
                state.requested += block_size;
            }
        }

        state.read_message();
    }

    return state.buf;
}

pub fn check_integrity(pw: &PieceWork, buf: &Vec<u8>) -> bool {
    let mut hasher = Sha1::new();
    hasher.input(buf);
    let sum_hex = hasher.result();

    for (index, item) in sum_hex.as_slice().iter().enumerate() {
        if pw.hash[index] != *item {
            return false;
        }
    }

    true
}

pub fn start_download_worker(
    p: Peer,
    info_hash: &[u8; 20],
    work_snd: Sender<p2p::PieceWork>,
    work_rcv: Receiver<p2p::PieceWork>,
    result_snd: Sender<p2p::PieceResult>,
) {
    let mut this_thread_client: client::Client;
    let peer_id: [u8; 20] = PEER_ID_STRING.as_bytes().try_into().unwrap();
    let peer_ip = p.ip;
    match client::new(p, peer_id, *info_hash) {
        Ok(client) => {
            this_thread_client = client;
            println!("success in completing handshake and unchoking");
            // this_thread_client.send_unchoke();
            // this_thread_client.send_interested();

            // for piece in work_rcv.recv() {
            //     if !this_thread_client.bitfield.has_piece(piece.index) {
            //         work_snd.send(piece).unwrap();
            //         continue;
            //     }

            //     let buf = attempt_download_piece(&mut this_thread_client, piece);
            //     // TODO: handle error, in which case put piece back on work queue (with work_snd.send)

            //     if !check_integrity(&piece, &buf) {
            //         println!("Piece {} failed integrity check", piece.index);
            //         work_snd.send(piece).unwrap();
            //         continue;
            //     }

            //     this_thread_client.send_have(piece.index);
            //     let piece_result = PieceResult {
            //         index: piece.index,
            //         buf: buf.clone(),
            //     };
            //     result_snd.send(piece_result).unwrap();
            // }
        }
        Err(_) => {
            println!("DROP: peer ip: {}", peer_ip);
        }
    }
}
