extern crate serde;
extern crate serde_bencode;
#[macro_use]
extern crate serde_derive;

use serde_bencode::de;
use std::io::{self, Read};
use std::sync::mpsc;
use std::thread;

mod client;
mod handshake;
mod p2p;
mod torrent;

pub static PEER_ID: &str = "kjh29409k8hj0wgej6c1";

fn main() {
    let stdin = io::stdin();
    let mut buffer = Vec::new();
    let mut handle = stdin.lock();
    let bencode_torrent;
    match handle.read_to_end(&mut buffer) {
        Ok(_) => match de::from_bytes::<torrent::BencodeTorrent>(&buffer) {
            Ok(t) => {
                bencode_torrent = t;
                torrent::render_bencode_torrent(&bencode_torrent);
            }
            Err(e) => panic!("ERROR: {:?}", e),
        },
        Err(e) => panic!("ERROR: {:?}", e),
    }

    let our_torrent = torrent::new_torrent(&bencode_torrent);

    // println!("\nTorrent struct:");
    our_torrent.render_torrent();

    let url = our_torrent.build_tracker_url().unwrap();
    // println!("\nURL: {}", url);

    // get tracker response (http get)
    let mut res = reqwest::blocking::get(&url).unwrap();
    // println!("{:#?}", res);

    // extract response body into resp_buffer
    let mut resp_buffer = Vec::new();
    let copy_result = res.copy_to(&mut resp_buffer);
    match copy_result {
        Ok(_) => (),
        Err(e) => panic!(e),
    }

    // deserialize tracker response into bencode struct
    let bencode_tracker_resp;
    match de::from_bytes::<torrent::BencodeTrackerResp>(&resp_buffer) {
        Ok(t) => {
            bencode_tracker_resp = t;
            println!("{:?}", bencode_tracker_resp);
            println!();
        }
        Err(e) => panic!("ERROR: {:?}", e),
    }

    // load peers into a vec of Peer structs
    let peers = bencode_tracker_resp.get_peers().unwrap();
    println!("Numer of peers found: {}", peers.len());
    // println!("{:?}", peers);
    println!();

    let (tx, rx) = mpsc::channel();

    let mut counter: usize = 0;
    for p in peers {
        counter += 1;
        // if counter > 10 {
        //     continue;
        // }
        let tx_p = mpsc::Sender::clone(&tx);
        let info_hash = our_torrent.info_hash;
        thread::spawn(move || {
            // dial peer tcp
            // println!("connecting to peer with IP: {}", p.ip);
            p2p::start_download_worker(p, &info_hash, tx_p);
            println!("after starting thread: {}", counter);
        });
    }

    for received in rx {
        println!("Got from channel: {}", received);
    }

    println!("exit program");
}
