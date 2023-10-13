use bit_tor::bencode::Bencode;
use bit_tor::MetaInfo;
use bit_tor::Peer;
use bit_tor::escape_u8_slice;
use core::panic;
use std::net::TcpStream;
use reqwest;
use std::collections::BTreeMap;
use std::env;
use std::fs;
use std::io::prelude::*;
use std::error::Error;
use std::io::BufReader;

// use std::thread;
// use std::time::Duration;

fn main() -> Result<(),Box<dyn Error>> {
    let args: Vec<String> = env::args().collect();
    let file = fs::File::open(
        args.get(1)
            .expect("No file supplied in command line invocation"),
    )?;
    let peer_id = make_peer_id();
    let (root_dict, info_bencoded) = read_torrent(file)?;
    let hashed_info = sha1_smol::Sha1::from(info_bencoded).digest().bytes();
    let meta_info = MetaInfo::construct_from_dict_v1(root_dict, hashed_info);
    let response = MetaInfo::tracker_get(&meta_info, peer_id)?;
    println!("TRACKER RESPONSE: {}",escape_u8_slice(&response));
    let mut peers = Peer::get_peers(response)?;
    peers.iter().for_each(|p| println!("Socket: {:?}",p.handle.peer_addr()));
    let handshake = serialize_handshake(&meta_info, make_peer_id());
    for peer in peers.iter_mut() {
        peer.write_to_peer(&handshake.as_slice())?;
    }
    dbg!(peers.len());
    dbg!("After Writes");
    for peer in peers.iter_mut() {
        
    }

    Ok(())
}

fn read_handshake(peer:&Peer) -> std::io::Result<Vec<u8>>{
    
    Ok(Vec::new())
}



type DictAndBytes =  (BTreeMap<Vec<u8>, Bencode>, Vec<u8>);
fn read_torrent(mut file: fs::File) -> Result<(DictAndBytes), std::io::Error>  {
    let mut buf = Vec::with_capacity(1_000_000);
    let _bytes_read = file.read_to_end(&mut buf);
    let root_dict = match Bencode::decode_dispatch(&mut buf.iter().peekable())? {
        Bencode::Dict(d) => d,
        _ => panic!("Top level bencoded value is not a dictionary"),
    };
    let info_bencoded = root_dict
        .get("info".as_bytes())
        .expect("No info value")
        .encode_val();
    Ok((root_dict, info_bencoded))
}

fn make_peer_id() -> String {
    const PEER_TAG: [u8; 8] = *b"-AV0001-";
    use rand::Rng;
    let mut res = Vec::new();
    res.extend(PEER_TAG);
    let mut rng = rand::thread_rng();
    for _ in 0..12 {
        res.push(rng.gen_range(b'0'..=b'9'));
    }
    String::from_utf8(res).unwrap()
}

//len(info_hash) + len(peer_id) + 8 reserved bytes + 1 byte declaring length of protocol string
const BASE_HANDSHAKE_LENGTH: u8 = 20 + 20 + 8 + 1;
fn serialize_handshake(meta: &MetaInfo, peer_id: String) -> Vec<u8> {
    let pstr = "BitTorrent protocol".as_bytes();
    let pstr_len = pstr.len() as u8;
    let mut raw_bytes = Vec::<u8>::with_capacity((BASE_HANDSHAKE_LENGTH + pstr_len) as usize);
    raw_bytes.push(pstr_len);
    raw_bytes.extend(pstr);
    raw_bytes.extend([0u8; 8]); //Reserved Bytes
    raw_bytes.extend(meta.info_hash);
    raw_bytes.extend(peer_id.as_bytes());
    raw_bytes
}

