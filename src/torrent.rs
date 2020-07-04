use serde_bencode::ser;
use serde_bytes::ByteBuf;
use sha1::{Digest, Sha1};
use std::net::{IpAddr, Ipv4Addr};
use url::{form_urlencoded, ParseError};

use crate::p2p;

#[derive(Debug, Serialize, Deserialize)]
pub struct Node(String, i64);

#[derive(Debug, Serialize, Deserialize)]
pub struct File {
    path: Vec<String>,
    length: i64,
    #[serde(default)]
    md5sum: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct BencodeInfo {
    name: String,
    pieces: ByteBuf,
    #[serde(rename = "piece length")]
    piece_length: i64,
    #[serde(default)]
    md5sum: Option<String>,
    #[serde(default)]
    length: Option<i64>,
    #[serde(default)]
    files: Option<Vec<File>>,
    #[serde(default)]
    private: Option<u8>,
    #[serde(default)]
    path: Option<Vec<String>>,
    #[serde(default)]
    #[serde(rename = "root hash")]
    root_hash: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct BencodeTorrent {
    info: BencodeInfo,
    #[serde(default)]
    announce: Option<String>,
    #[serde(default)]
    nodes: Option<Vec<Node>>,
    #[serde(default)]
    encoding: Option<String>,
    #[serde(default)]
    httpseeds: Option<Vec<String>>,
    #[serde(default)]
    #[serde(rename = "announce-list")]
    announce_list: Option<Vec<Vec<String>>>,
    #[serde(default)]
    #[serde(rename = "creation date")]
    creation_date: Option<i64>,
    #[serde(rename = "comment")]
    comment: Option<String>,
    #[serde(default)]
    #[serde(rename = "created by")]
    created_by: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct BencodeTrackerResp {
    #[serde(default)]
    interval: i64,
    #[serde(default)]
    peers: ByteBuf,
}

pub struct Torrent {
    pub announce: String,
    pub name: String,
    pub length: i64,
    pub info_hash: [u8; 20],
    pub piece_length: i64,
    pub piece_hashes: Vec<[u8; 20]>,
}

#[derive(Debug, Clone)]
pub enum InvalidTorrentError {
    WrongNumberOfPieces,
}

#[derive(Debug, Clone)]
pub enum TrackerError {
    InvalidPeerResponse,
}

impl BencodeTrackerResp {
    pub fn get_peers(&self) -> Result<Vec<p2p::Peer>, TrackerError> {
        let mut final_peers: Vec<p2p::Peer> = vec![];

        let peer_size = 6; // 4 for IP, 2 for port
        let num_peers = self.peers.len() / peer_size;
        if self.peers.len() % peer_size != 0 {
            return Err(TrackerError::InvalidPeerResponse);
        }

        for i in 0..num_peers {
            let offset = i * peer_size;

            let ip_slice = &self.peers[offset..offset + 4];
            let port_arr: [u8; 2] = [self.peers[offset + 4], self.peers[offset + 5]];
            let port = u16::from_be_bytes(port_arr);

            let newpeer = p2p::Peer {
                ip: IpAddr::V4(Ipv4Addr::new(
                    ip_slice[0],
                    ip_slice[1],
                    ip_slice[2],
                    ip_slice[3],
                )),
                port,
            };
            final_peers.push(newpeer);
        }

        Ok(final_peers)
    }
}

impl BencodeInfo {
    pub fn calculate_info_hash(&self) -> [u8; 20] {
        let info_bencoded = ser::to_bytes(&self).unwrap();
        let mut hasher = Sha1::new();
        hasher.input(info_bencoded);
        let sum_hex = hasher.result();
        let mut sum_bytes: [u8; 20] = Default::default();
        sum_bytes.copy_from_slice(sum_hex.as_slice());
        sum_bytes
    }

    pub fn split_piece_hashes(&self) -> Result<Vec<[u8; 20]>, InvalidTorrentError> {
        // handle info.pieces length not being divided by 20
        if self.pieces.len() % 20 != 0 {
            return Err(InvalidTorrentError::WrongNumberOfPieces);
        }

        let mut hash_list: Vec<[u8; 20]> = Vec::new();
        let mut hash = [0u8; 20];
        let mut current_index: usize = 0;
        for piece in self.pieces.iter() {
            if current_index >= 20 {
                hash_list.push(hash);
                hash = [0u8; 20];
                current_index = 0;
            }
            hash[current_index] = *piece;
            current_index += 1;
        }
        hash_list.push(hash);

        // println!("hash_list: {:?}", hash_list);
        Ok(hash_list)
    }
}

impl Torrent {
    pub fn build_tracker_url(&self) -> Result<String, ParseError> {
        let infohash_urlencoded: String =
            form_urlencoded::byte_serialize(&self.info_hash).collect();

        let peer_id_urlencoded: String =
            form_urlencoded::byte_serialize(p2p::PEER_ID_STRING.as_bytes()).collect();

        let querystring = format!(
            "?info_hash={info_hash}&peer_id={peer_id}&port={port}&uploaded={uploaded}&downloaded={downloaded}&compact={compact}&left={left}",
            info_hash=infohash_urlencoded,
            peer_id=peer_id_urlencoded,
            port="6881",
            uploaded="0",
            downloaded="0",
            left=self.length,
            compact="1",
        );
        let mut final_url = self.announce.to_owned();
        final_url.push_str(&querystring);
        Ok(final_url)
    }

    pub fn render_torrent(&self) {
        println!("announce: {}", self.announce);
        println!("name: {}", self.name);
        println!("length: {}", self.length);
        println!("info_hash: {:?}", self.info_hash);
        println!("piece_length: {}", self.piece_length);
        // println!("piece_hashes: {:?}", self.piece_hashes);
    }

    pub fn calculate_bounds_for_piece(&self, index: i64) -> (i64, i64) {
        let begin: i64 = index * self.piece_length;
        let mut end: i64 = begin + self.piece_length;
        if end > self.length {
            end = self.length;
        }
        return (begin, end);
    }

    pub fn calculate_piece_size(&self, index: i64) -> i64 {
        let (begin, end) = self.calculate_bounds_for_piece(index);
        return end - begin;
    }
}

pub fn new_torrent(bencode_torrent: &BencodeTorrent) -> Torrent {
    Torrent {
        announce: bencode_torrent.announce.as_ref().unwrap().to_string(),
        name: bencode_torrent.info.name.clone(),
        length: bencode_torrent.info.length.unwrap(),
        info_hash: bencode_torrent.info.calculate_info_hash(),
        piece_length: bencode_torrent.info.piece_length,
        piece_hashes: bencode_torrent.info.split_piece_hashes().unwrap(),
    }
}

pub fn render_bencode_torrent(torrent: &BencodeTorrent) {
    println!("name:\t\t{}", torrent.info.name);
    println!("announce:\t{:?}", torrent.announce);
    println!("nodes:\t\t{:?}", torrent.nodes);
    if let &Some(ref al) = &torrent.announce_list {
        for a in al {
            println!("announce list:\t{}", a[0]);
        }
    }
    println!("httpseeds:\t{:?}", torrent.httpseeds);
    println!("creation date:\t{:?}", torrent.creation_date);
    println!("comment:\t{:?}", torrent.comment);
    println!("created by:\t{:?}", torrent.created_by);
    println!("encoding:\t{:?}", torrent.encoding);
    println!("piece length:\t{:?}", torrent.info.piece_length);
    println!("private:\t{:?}", torrent.info.private);
    println!("root hash:\t{:?}", torrent.info.root_hash);
    println!("md5sum:\t\t{:?}", torrent.info.md5sum);
    println!("path:\t\t{:?}", torrent.info.path);
    if let &Some(ref files) = &torrent.info.files {
        for f in files {
            println!("file path:\t{:?}", f.path);
            println!("file length:\t{}", f.length);
            println!("file md5sum:\t{:?}", f.md5sum);
        }
    }
}
