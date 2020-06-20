use std::io::{Read, Write};
use std::net::{Shutdown, SocketAddr, TcpStream};
use std::time::Duration;

use crate::bitfield;
use crate::handshake;
use crate::message;
use crate::p2p;

#[derive(Debug, Clone)]
pub enum ClientError {
    ConnectionFailure,
    WrongType,
}

// Client is a TCP connection with a peer
pub struct Client {
    pub conn: TcpStream,
    pub choked: bool,
    pub bitfield: bitfield::Bitfield,
    pub peer: p2p::Peer,
    pub info_hash: [u8; 20],
    pub peer_id: [u8; 20],
}

pub fn new(p: p2p::Peer, peer_id: [u8; 20], info_hash: [u8; 20]) -> Result<Client, ClientError> {
    println!("Connecting to peer {}", p.ip);
    let addr = SocketAddr::new(p.ip, p.port);
    match TcpStream::connect_timeout(&addr, Duration::new(5, 0)) {
        Ok(mut stream) => {
            println!("Successfully connected to peer {}", addr);
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
            // handshake is 68 bytes
            let mut data: Vec<u8> = vec![0u8; 68];
            match stream.read_exact(&mut data) {
                Ok(_) => {
                    // println!("handshake_response raw from: {}, data: {:?}", addr, data);
                    let handshake_response = handshake::read_handshake(&data);
                    if handshake_response.pstr.len() != 0 {
                        // println!("handshake_response success, len: {}", data.len());
                        // println!(
                        //     "handshake_response struct from: {}, data: {:?}",
                        //     addr, handshake_response
                        // );
                        let bitfield = receive_bitfield(&mut stream).unwrap();
                        Ok(Client {
                            conn: stream,
                            choked: true,
                            peer: p,
                            bitfield,
                            info_hash,
                            peer_id,
                        })
                    } else {
                        // println!("handshake_response, failed, equals 0, for: {}", addr);
                        Err(ClientError::ConnectionFailure)
                    }
                }
                Err(e) => {
                    // println!("Failed to receive data from: {}, err: {}", addr, e);
                    stream.shutdown(Shutdown::Both).unwrap();
                    Err(ClientError::ConnectionFailure)
                }
            }
        }
        Err(e) => {
            // println!("Could not connect to: {}, err: {}", addr, e);
            Err(ClientError::ConnectionFailure)
        }
    }
}

fn receive_bitfield(stream: &mut TcpStream) -> Result<bitfield::Bitfield, ClientError> {
    let mut buf: Vec<u8> = vec![0, 0, 0, 0];
    match stream.read_exact(&mut buf) {
        Ok(_) => {
            let msg = message::read_message(buf);
            if msg.id != message::MSG_BITFIELD {
                println!("Expected bitfield but got type: {}", msg.id);
                return Err(ClientError::WrongType);
            }
            Ok(bitfield::Bitfield { array: msg.payload })
        }
        Err(e) => {
            println!("Shutting down: error: {}", e);
            stream.shutdown(Shutdown::Both).unwrap();
            Err(ClientError::ConnectionFailure)
        }
    }
}

impl Client {
    pub fn read(&mut self) -> Option<message::Message> {
        let mut msg_length = vec![0u8; 4];
        match self.conn.read_exact(&mut msg_length) {
            Ok(_) => {
                // check if keep-alive message
                let msg_length_arr = [msg_length[0], msg_length[1], msg_length[2], msg_length[3]];
                let msg_length_u32 = u32::from_be_bytes(msg_length_arr);
                if msg_length_u32 == 0 {
                    return None;
                }

                let mut msg_data = vec![0u8; msg_length_u32 as usize];
                match self.conn.read_exact(&mut msg_data) {
                    Ok(_) => {
                        let mut msg_data_full: Vec<u8> = vec![0u8; 4 + msg_length_u32 as usize];

                        // copy length's 4 bytes into msg_data_full
                        for (index, item) in msg_length.iter().enumerate() {
                            msg_data_full[index] = *item;
                        }

                        // copy data bytes into msg_data_full, after length's bytes
                        for (index, item) in msg_data.iter().enumerate() {
                            msg_data_full[4 + index] = *item;
                        }

                        // return msg into message struct
                        let msg_struct = message::read_message(msg_data_full);
                        Some(msg_struct)
                    }
                    Err(e) => {
                        println!("Unable to read message content: {}", e);
                        None
                    }
                }
            }
            Err(e) => {
                println!("Unable to read message length: {}", e);
                None
            }
        }
    }

    pub fn send_unchoke(&mut self) {
        let msg = message::Message {
            id: message::MSG_UNCHOKE,
            payload: vec![],
        };
        self.conn.write(&msg.serialize()).unwrap();
    }

    pub fn send_interested(&mut self) {
        let msg = message::Message {
            id: message::MSG_INTERESTED,
            payload: vec![],
        };
        self.conn.write(&msg.serialize()).unwrap();
    }
}
