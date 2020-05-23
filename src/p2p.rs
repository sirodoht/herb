use std::io::{self, Read, Write};
use std::net::{SocketAddr, TcpStream};
use std::{convert::TryInto, time::Duration};

use crate::handshake;
use crate::torrent::Peer;

pub fn start_download_worker(addr: SocketAddr, info_hash: &[u8; 20]) {
    // start a tcp connection with peer
    match TcpStream::connect_timeout(&addr, Duration::new(3, 0)) {
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
            let handshake = handshake::new_handshake(*info_hash, *conv_to_20(peer_id_transformed));
            println!("handshake: {:?}", handshake);

            let handshake_serialized = handshake.serialize();
            println!("handshake_serialized: {:?}", handshake_serialized);
            stream
                .set_write_timeout(Some(Duration::new(5, 0)))
                .expect("cannot set write timeout, lol");

            stream
                .write(&handshake_serialized)
                .expect("handshake response error");
            println!("sent handshake");

            let mut data: Vec<u8> = Default::default();
            match stream.read_to_end(&mut data) {
                Ok(_) => {
                    println!("handshake_response raw: {:?}", data);
                    let handshake_response = handshake::read_handshake(&data);
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
