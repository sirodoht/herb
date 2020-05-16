extern crate serde;
extern crate serde_bencode;
#[macro_use]
extern crate serde_derive;
// extern crate serde_bytes;
// extern crate sha1;
// extern crate url;

use serde_bencode::de;
use std::io::{self, Read, Write};
use std::net::{SocketAddr, TcpStream};
use std::{convert::TryInto, time::Duration};

mod handshake;
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
    println!(
        "connecting to peer with IP: {}:{}",
        peers[0].ip, peers[0].port
    );
    // start a tcp connection with peers
    match TcpStream::connect_timeout(&addr, Duration::new(30, 0)) {
        Ok(mut stream) => {
            println!("Successfully connected to peer");

            fn conv_to_20(slice: &[u8]) -> &[u8; 20] {
                slice
                    .try_into()
                    .expect("could not fit peer id into 20 bytes")
            }

            println!("crate::PEER_ID.as_bytes(): {:?}", crate::PEER_ID.as_bytes());
            let peer_id_transformed: &[u8] = crate::PEER_ID.as_bytes();
            println!("peer_id_transformed: {:?}", peer_id_transformed);
            println!(
                "*conv_to_20(peer_id_transformed): {:?}",
                *conv_to_20(peer_id_transformed)
            );
            let handshake =
                handshake::new_handshake(our_torrent.info_hash, *conv_to_20(peer_id_transformed));
            println!("handshake: {:?}", handshake);

            let handshake_serialized = handshake.serialize();
            println!("handshake_serialized: {:?}", handshake_serialized);
            stream
                .set_write_timeout(Some(Duration::new(30, 0)))
                .expect("cannot set write timeout, lol");

            stream
                .write(&handshake_serialized)
                .expect("handshake response error");
            println!("sent handshake");

            let mut data: Vec<u8> = Default::default();
            match stream.read_to_end(&mut data) {
                Ok(_) => {
                    println!("handshake_response raw: {:?}", data);
                    let handshake_response = handshake::read_handshake(data);
                    println!("handshake_response struct: {:?}", handshake_response);
                }
                Err(e) => {
                    println!("Failed to receive data: {}", e);
                }
            }
        }
        Err(e) => {
            println!("Failed to connect: {}", e);
        }
    }
}
