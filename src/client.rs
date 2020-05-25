use std::io::{Read, Write};
use std::net::{Shutdown, SocketAddr, TcpStream};
use std::time::Duration;

use crate::handshake;
use crate::p2p;

#[derive(Debug, Clone)]
pub enum ClientError {
    ConnectionFailure,
}

// Client is a TCP connection with a peer
pub struct Client {
    conn: TcpStream,
    choked: bool,
    // bitfield: Bitfield,
    peer: p2p::Peer,
    info_hash: [u8; 20],
    peer_id: [u8; 20],
}

pub fn new(p: p2p::Peer, peer_id: [u8; 20], info_hash: [u8; 20]) -> Result<Client, ClientError> {
    println!("Connecting to peer {}", p.ip);
    let addr = SocketAddr::new(p.ip, p.port);
    match TcpStream::connect_timeout(&addr, Duration::new(5, 0)) {
        Ok(mut stream) => {
            println!("Successfully connected to peer {}", addr);
            println!("after send_peer_handshake");
            let handshake = handshake::new_handshake(info_hash, peer_id);
            // println!("handshake: {:?}", handshake);

            let handshake_serialized = handshake.serialize();
            // println!("handshake serialized:");
            // println!("{:?}", handshake_serialized);

            stream
                .set_write_timeout(Some(Duration::new(5, 0)))
                .expect("cannot set write timeout, lol");

            stream
                .write_all(&handshake_serialized)
                .expect("handshake response error");

            stream
                .set_read_timeout(Some(Duration::new(5, 0)))
                .expect("could not set read timeout :(");

            println!("send handshake to peer {}", addr);
            let mut data: Vec<u8> = Vec::new();
            match stream.read_to_end(&mut data) {
                Ok(_) => {
                    println!("handshake_response raw from: {}, data: {:?}", addr, data);
                    if data.len() != 0 {
                        let handshake_response = handshake::read_handshake(&data);
                        println!("handshake_response success, len: {}", data.len());
                        println!(
                            "handshake_response struct from: {}, data: {:?}",
                            addr, handshake_response
                        );
                        Ok(Client {
                            conn: stream,
                            choked: true,
                            peer: p,
                            info_hash,
                            peer_id,
                        })
                    } else {
                        println!("handshake_response, failed, equals 0, for: {}", addr);
                        Err(ClientError::ConnectionFailure)
                    }
                }
                Err(e) => {
                    println!("Failed to receive data from: {}, err: {}", addr, e);
                    stream.shutdown(Shutdown::Both).unwrap();
                    Err(ClientError::ConnectionFailure)
                }
            }
        }
        Err(e) => {
            println!("Could not connect to: {}, err: {}", addr, e);
            Err(ClientError::ConnectionFailure)
        }
    }
}

fn receive_bitfield() {}
