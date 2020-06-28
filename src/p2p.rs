use crossbeam::channel::{Receiver, Sender};
use sha1::{Digest, Sha1};
use std::convert::TryInto;
use std::net::IpAddr;
use std::time::Duration;

use crate::client;
use crate::message;
use crate::p2p;

pub static PEER_ID_STRING: &str = "kjh29409k8hj0wgej6c1";

#[derive(Debug, Clone)]
pub enum PieceError {
    DownloadFailure,
    MessageParsingFailure,
}

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
    pub fn read_message(&mut self) -> Option<PieceError> {
        match self.client.read() {
            None => Some(PieceError::MessageParsingFailure),
            Some(msg) => {
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
                None
            }
        }
    }
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

pub fn attempt_download_piece(
    c: &mut client::Client,
    pw: PieceWork,
) -> Result<Vec<u8>, PieceError> {
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

        match state.read_message() {
            None => {
                return Ok(state.buf);
            }
            Some(error) => {
                println!("Failed to read new message, error: {:?}", error);
                return Err(error);
            }
        }
    }

    Ok(state.buf)
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
            this_thread_client.send_unchoke();
            this_thread_client.send_interested();
            // println!(
            //     "success in completing handshaking/unchoking with IP: {}",
            //     peer_ip
            // );

            println!("{}: ready for pieces of work", peer_ip);
            for piece in work_rcv.recv() {
                println!("{}: received new work with index: {}", peer_ip, piece.index);
                if !this_thread_client.bitfield.has_piece(piece.index) {
                    work_snd.send(piece).unwrap();
                    println!("bitfield failure, ip: {}", peer_ip);
                    continue;
                }

                println!(
                    "{}: bitfield success, attempt piece: {}",
                    peer_ip, piece.index
                );
                let buf_result = attempt_download_piece(&mut this_thread_client, piece);
                match buf_result {
                    Ok(buf) => {
                        if !check_integrity(&piece, &buf) {
                            println!("{}: Piece {} failed integrity check", peer_ip, piece.index);
                            work_snd.send(piece).unwrap();
                            println!("putting back work, piece: {}", piece.index);
                            continue;
                        }
                        println!(
                            "{}: Piece {} integrity check success!",
                            peer_ip, piece.index
                        );

                        this_thread_client.send_have(piece.index);
                        println!("{}: Piece {} send have", peer_ip, piece.index);
                        let piece_result = PieceResult {
                            index: piece.index,
                            buf: buf.clone(),
                        };
                        result_snd.send(piece_result).unwrap();
                        println!("{}: Piece {} send result !", peer_ip, piece.index);
                    }
                    Err(e) => {
                        println!("Exiting, attempt at download failed: {:?}", e);
                        work_snd.send(piece).unwrap(); // put piece back on the queue
                        println!("Putting back work, piece: {}", piece.index);
                        continue;
                    }
                }
                println!("{}: blocking for new work", peer_ip);
            }
        }
        Err(e) => {
            println!("{}: DROPPED, with error: {:?}", peer_ip, e);
        }
    }

    println!("{}: end", peer_ip);
}
