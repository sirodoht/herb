extern crate serde;
extern crate serde_bencode;
#[macro_use]
extern crate serde_derive;

use serde_bencode::de;
use std::io::{self, Read};
use std::{thread, time};

// use crossbeam::channel::unbounded;

mod bitfield;
mod client;
mod handshake;
mod message;
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

    let (work_snd, work_rcv) = crossbeam::unbounded();
    for (index, piece_hash) in our_torrent.piece_hashes.iter().enumerate() {
        let length = our_torrent.calculate_piece_size(index as i64);
        let piece_work = p2p::PieceWork {
            index: index as i64,
            hash: *piece_hash,
            length,
        };
        work_snd.send(piece_work);
    }

    let (result_snd, result_rcv) = crossbeam::unbounded();

    for p in peers {
        let (work_snd_peer, work_rcv_peer) = (work_snd.clone(), work_rcv.clone());
        let (result_snd_peer, result_rcv_peer) = (result_snd.clone(), result_rcv.clone());
        let info_hash = our_torrent.info_hash;
        crossbeam::scope(|s| {
            s.spawn(|_| {
                // dial peer tcp
                // println!("connecting to peer with IP: {}", p.ip);
                p2p::start_download_worker(
                    p,
                    &info_hash,
                    work_snd_peer,
                    work_rcv_peer,
                    result_snd_peer,
                    result_rcv_peer,
                );
                // println!("after starting thread: {}", counter);
            });
        })
        .unwrap();
    }

    // Collect results into a buffer until full
    let mut buf = vec![0u8; our_torrent.length as usize];
    let mut done_pieces = 0;
    while done_pieces < our_torrent.piece_hashes.len() {
        let res = result_rcv.recv().unwrap();
        let (begin, end) = our_torrent.calculate_bounds_for_piece(res.index);

        // copy data from res.buf to buf[begin:end]
        let mut cache = begin as usize;
        for item in res.buf {
            buf[cache] = item;
            cache += 1;
        }

        done_pieces += 1;
        println!("Downloaded piece {}", done_pieces);
    }

    println!("exit program");
}
