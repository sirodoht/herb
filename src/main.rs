extern crate serde;
extern crate serde_bencode;
#[macro_use]
extern crate serde_derive;
extern crate base64;
extern crate serde_bytes;
extern crate sha1;
extern crate url;

use serde_bencode::de;
use serde_bencode::ser;
use serde_bytes::ByteBuf;
use std::io::{self, Read};
use url::{form_urlencoded, ParseError, Url};

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
    piece_hashes: Vec<[u8; 20]>,
}

#[derive(Debug, Clone)]
enum InvalidTorrentError {
    WrongNumberOfPieces,
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

    fn split_piece_hashes(&self) -> Result<Vec<[u8; 20]>, InvalidTorrentError> {
        // handle info.pieces length not being divided by 20
        if self.pieces.len() % 20 != 0 {
            return Err(InvalidTorrentError::WrongNumberOfPieces);
        }

        let mut hash_list: Vec<[u8; 20]> = Vec::new();
        let mut hash: [u8; 20] = Default::default();
        let mut current_index: usize = 0;
        for piece in self.pieces.iter() {
            // add piece in hash unless hash is full
            if current_index < 20 {
                hash[current_index] = *piece;
                current_index += 1;
            } else {
                // if hash full, push hash into hash_list
                // and consider hash empty (by reverting index to 0)
                hash_list.push(hash);
                current_index = 0;
            }
        }
        Ok(hash_list)
    }
}

impl Torrent {
    fn build_tracker_url(&self) -> Result<String, ParseError> {
        let querystring = format!(
            "/?info_hash={info_hash}&peer_id={peer_id}&port={port}&uploaded={uploaded}&downloaded={downloaded}&compact={compact}&left={left}",
            info_hash=base64::encode(self.info_hash),
            peer_id="-TR2940-k8hj0wgej6c1",
            port="6881",
            uploaded="0",
            downloaded="0",
            compact="1",
            left=self.length,
        );
        let mut final_url = self.announce.to_owned();
        final_url.push_str(&querystring);
        Ok(final_url)
    }
}

fn new_torrent(bencode_torrent: &BencodeTorrent) -> Torrent {
    let torrent = Torrent {
        announce: bencode_torrent.announce.as_ref().unwrap().to_string(),
        name: bencode_torrent.info.name.clone(),
        length: bencode_torrent.info.length.unwrap(),
        info_hash: bencode_torrent.info.calculate_info_hash(),
        piece_length: bencode_torrent.info.piece_length,
        piece_hashes: bencode_torrent.info.split_piece_hashes().unwrap(),
    };
    torrent
}

fn render_torrent(torrent: &Torrent) {
    println!("announce: {}", torrent.announce);
    println!("name: {}", torrent.name);
    println!("length: {}", torrent.length);
    println!("info_hash: {:?}", torrent.info_hash);
    println!("piece_length: {}", torrent.piece_length);
    // println!("piece_hashes: {:?}", torrent.piece_hashes);
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

    let url = torrent.build_tracker_url().unwrap();
    println!("\nURL: {}", url);
}
