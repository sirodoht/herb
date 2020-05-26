use std::convert::TryInto;
use std::net::IpAddr;
use std::sync::mpsc::Sender;

use crate::client;

pub static PEER_ID_STRING: &str = "kjh29409k8hj0wgej6c1";

#[derive(Debug)]
pub struct Peer {
    pub ip: IpAddr,
    pub port: u16,
}

pub fn start_download_worker(p: Peer, info_hash: &[u8; 20], tx_p: Sender<String>) {
    let this_thread_client: client::Client;
    let peer_id: [u8; 20] = PEER_ID_STRING.as_bytes().try_into().unwrap();
    let peer_ip = p.ip;
    match client::new(p, peer_id, *info_hash) {
        Ok(client) => {
            this_thread_client = client;
        }
        Err(_) => {
            println!("DROP: peer ip: {}", peer_ip);
        }
    }
}
