extern crate serde;
extern crate serde_bencode;
#[macro_use]
extern crate serde_derive;
// extern crate serde_bytes;
// extern crate sha1;
// extern crate url;

use serde_bencode::de;
use std::io::{self, Read};
use std::net::{SocketAddr, TcpStream};
use std::time::Duration;

mod torrent;

pub static PEER_ID: &str = "-TR2940-k8hj0wgej6c1";

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

    println!("\nTorrent struct:");
    our_torrent.render_torrent();

    let url = our_torrent.build_tracker_url().unwrap();
    println!("\nURL: {}", url);

    // get tracker response
    let mut res = reqwest::blocking::get(&url).unwrap();
    println!("{:#?}", res);

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
    println!("{:?}", peers);
    println!();

    // dial peer tcp
    let addr = SocketAddr::new(peers[0].ip, peers[0].port);
    // start a tcp connection with peers
    match TcpStream::connect_timeout(&addr, Duration::new(3, 0)) {
        Ok(mut stream) => {
            println!("Successfully connected to peer");

            // let msg = b"Hello!";
            // let handshake = handshake::new_handshake(torrent.info_hash, &PEER_ID);

            // stream.write(msg).unwrap();
            // println!("Sent Hello, awaiting reply...");

            // let mut data = [0 as u8; 6]; // using 6 byte buffer
            // match stream.read_exact(&mut data) {
            //     Ok(_) => {
            //         if &data == msg {
            //             println!("Reply is ok!");
            //         } else {
            //             let text = from_utf8(&data).unwrap();
            //             println!("Unexpected reply: {}", text);
            //         }
            //     }
            //     Err(e) => {
            //         println!("Failed to receive data: {}", e);
            //     }
            // }
        }
        Err(e) => {
            println!("Failed to connect: {}", e);
        }
    }
}
