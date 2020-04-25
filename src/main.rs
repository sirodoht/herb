extern crate serde;
extern crate serde_bencode;
#[macro_use]
extern crate serde_derive;
extern crate serde_bytes;
extern crate sha1;

use serde_bencode::de;
use serde_bencode::ser;
use serde_bytes::ByteBuf;
use std::io::{self, Read};

#[derive(Debug, Serialize, Deserialize)]
struct Node(String, i64);

#[derive(Debug, Serialize, Deserialize)]
struct File {
    path: Vec<String>,
    length: i64,
    #[serde(default)]
    md5sum: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
struct BencodeInfo {
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
struct BencodeTorrent {
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

struct Torrent {
    announce: String,
    name: String,
    length: i64,
    info_hash: [u8; 20],
    piece_length: i64,
    // piece_hashes: Vec<[u8; 20]>,
}

impl BencodeInfo {
    fn calculate_info_hash(&self) -> [u8; 20] {
        let info_bencoded = ser::to_bytes(&self).unwrap();
        let mut info_hasher = sha1::Sha1::new();
        info_hasher.update(&info_bencoded);
        let mut info_hashed: [u8; 20] = Default::default();
        let info_hashed_string = info_hasher.digest().to_string();
        let info_hashed_slice = info_hashed_string.as_bytes();
        info_hashed.copy_from_slice(&info_hashed_slice[0..20]);
        info_hashed
    }
}

fn new_torrent(bencode_torrent: &BencodeTorrent) -> Torrent {
    let torrent = Torrent {
        announce: bencode_torrent.announce.as_ref().unwrap().to_string(),
        name: bencode_torrent.info.name.clone(),
        length: bencode_torrent.info.length.unwrap(),
        info_hash: bencode_torrent.info.calculate_info_hash(),
        piece_length: bencode_torrent.info.piece_length,
    };
    torrent
}

fn render_torrent(torrent: &Torrent) {
    println!("announce: {}", torrent.announce);
    println!("name: {}", torrent.name);
    println!("length: {}", torrent.length);
    println!("info_hash: {:?}", torrent.info_hash);
}

fn render_bencode_torrent(torrent: &BencodeTorrent) {
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

fn main() {
    let stdin = io::stdin();
    let mut buffer = Vec::new();
    let mut handle = stdin.lock();
    let bencode_torrent;
    match handle.read_to_end(&mut buffer) {
        Ok(_) => match de::from_bytes::<BencodeTorrent>(&buffer) {
            Ok(t) => {
                bencode_torrent = t;
                render_bencode_torrent(&bencode_torrent);
            }
            Err(e) => panic!("ERROR: {:?}", e),
        },
        Err(e) => panic!("ERROR: {:?}", e),
    }

    let torrent = new_torrent(&bencode_torrent);

    println!("\nTorrent struct:");
    render_torrent(&torrent);
}
