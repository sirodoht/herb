use std::io::{Read, Write};
use std::net::{SocketAddr, TcpStream};
use std::{convert::TryInto, time::Duration};

use crate::handshake;

fn conv_to_20(slice: &[u8]) -> &[u8; 20] {
    slice
        .try_into()
        .expect("could not fit peer id into 20 bytes")
}

pub fn send_peer_handshake(addr: SocketAddr, stream: &mut TcpStream, info_hash: &[u8; 20]) {
    let peer_id_transformed: &[u8] = crate::PEER_ID.as_bytes();
    let handshake = handshake::new_handshake(*info_hash, *conv_to_20(peer_id_transformed));
    println!("handshake: {:?}", handshake);

    let handshake_serialized = handshake.serialize();
    println!("handshake serialized:");
    println!("{:?}", handshake_serialized);

    stream
        .set_write_timeout(Some(Duration::new(5, 0)))
        .expect("cannot set write timeout, lol");

    stream
        .write_all(&handshake_serialized)
        .expect("handshake response error");

    println!("sent handshake");

    stream
        .set_read_timeout(Some(Duration::new(5, 0)))
        .expect("could not set read timeout :(");

    println!("start reading from peer {}", addr);
    let mut data: Vec<u8> = Vec::new();
    match stream.read_to_end(&mut data) {
        Ok(_) => {
            println!("handshake_response raw: {:?}", data);
            if data.len() != 0 {
                let handshake_response = handshake::read_handshake(&data);
                println!("handshake_response struct: {:?}", handshake_response);
            } else {
                println!("handshake_response equals 0");
            }
        }
        Err(e) => {
            println!("Failed to receive data: {}", e);
        }
    }
}

pub fn start_download_worker(addr: SocketAddr, info_hash: &[u8; 20]) {
    println!("Connecting to peer {}", addr);
    match TcpStream::connect_timeout(&addr, Duration::new(5, 0)) {
        Ok(mut stream) => {
            println!("Successfully connected to peer {}", addr);

            send_peer_handshake(addr, &mut stream, info_hash);
        }
        Err(e) => {
            println!("Failed to connect: {}", e);
        }
    }
}
