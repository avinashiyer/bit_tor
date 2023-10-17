use bit_tor::bencode::Bencode;
use bit_tor::escape_u8_slice;
use bit_tor::vec_to_array;
use bit_tor::MetaInfo;
use bit_tor::Peer;
use core::panic;
use reqwest;
use std::collections::BTreeMap;
use std::env;
use std::error::Error;
use std::fs;
use std::io::prelude::*;
use std::io::BufReader;
use std::net::TcpStream;

// use std::thread;
// use std::time::Duration;

fn main() -> Result<(), Box<dyn Error>> {
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
    println!("TRACKER RESPONSE: {}", escape_u8_slice(&response));
    let mut peers = Peer::get_peers(response)?;
    peers
        .iter()
        .for_each(|p| println!("Socket: {:?}", p.socket));
    let handshake = serialize_handshake(&meta_info, make_peer_id());
    for peer in peers.iter_mut() {
        peer.write_to_peer(&handshake.as_slice())?;
    }
    dbg!(peers.len());
    dbg!("After Writes");
    // Drop peers with bad info hash
    peers.retain_mut(|peer: &mut Peer| {
        let peers_info_hash = read_handshake(peer).unwrap_or(Vec::new());
        hashed_info[..] == peers_info_hash
    });

    Ok(())
}

//  Handshake Structure:
//  [pstr_len][pstr][reserved][info_hash][peer_id]
//  [1]       [n]   [8]       [20]       [20]
fn read_handshake(peer: &mut Peer) -> std::io::Result<Vec<u8>> {
    let pstr_len = vec_to_array::<u8, 1>(Peer::loop_read(&mut peer.buf_reader, 1)?)[0];
    let bytes_to_read = (pstr_len + 49) as usize;
    let info_hash_start = (pstr_len + 8) as usize;
    let all_bytes: Vec<u8> = Peer::loop_read(&mut peer.buf_reader, bytes_to_read)?;
    Ok(all_bytes[info_hash_start..info_hash_start + 20].to_vec())
}

type DictAndBytes = (BTreeMap<Vec<u8>, Bencode>, Vec<u8>);
fn read_torrent(mut file: fs::File) -> Result<DictAndBytes, std::io::Error> {
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
const BASE_HANDSHAKE_LENGTH: usize = 20 + 20 + 8 + 1;
fn serialize_handshake(meta: &MetaInfo, peer_id: String) -> Vec<u8> {
    let pstr = "BitTorrent protocol".as_bytes();
    let pstr_len = pstr.len();
    let mut raw_bytes = Vec::<u8>::with_capacity(BASE_HANDSHAKE_LENGTH + pstr_len);
    raw_bytes.push(pstr_len as u8);
    raw_bytes.extend(pstr);
    raw_bytes.extend([0u8; 8]); //Reserved Bytes
    raw_bytes.extend(meta.info_hash);
    raw_bytes.extend(peer_id.as_bytes());
    raw_bytes
}
