use crossbeam::channel::{Receiver, Sender};
use std::convert::TryInto;
use std::net::IpAddr;

use crate::client;
use crate::p2p;

pub static PEER_ID_STRING: &str = "kjh29409k8hj0wgej6c1";

#[derive(Debug)]
pub struct Peer {
    pub ip: IpAddr,
    pub port: u16,
}

pub struct PieceWork {
    pub index: i64,
    pub hash: [u8; 20],
    pub length: i64,
}

pub struct PieceResult {
    pub index: i64,
    pub buf: Vec<u8>,
}

pub struct PieceProgress {
    pub index: i64,
    pub client: client::Client,
    pub buf: Vec<u8>,
    pub downloaded: i64,
    pub requested: i64,
    pub backlog: i64,
}

pub fn start_download_worker(
    p: Peer,
    info_hash: &[u8; 20],
    work_snd: Sender<p2p::PieceWork>,
    work_rcv: Receiver<p2p::PieceWork>,
    result_snd: Sender<p2p::PieceResult>,
    result_rcv: Receiver<p2p::PieceResult>,
) {
    let mut this_thread_client: client::Client;
    let peer_id: [u8; 20] = PEER_ID_STRING.as_bytes().try_into().unwrap();
    let peer_ip = p.ip;
    match client::new(p, peer_id, *info_hash) {
        Ok(client) => {
            this_thread_client = client;
            this_thread_client.send_unchoke();
            this_thread_client.send_interested();
            println!("success in completing handshake and unchoking");

            for piece in work_rcv.recv() {
                if !this_thread_client.bitfield.has_piece(piece.index) {
                    work_snd.send(piece);
                }
            }
        }
        Err(_) => {
            println!("DROP: peer ip: {}", peer_ip);
        }
    }
}
