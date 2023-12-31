use bit_tor::{bencode::Bencode, escape_u8_slice, vec_to_array, MetaInfo, Peer};

use core::panic;
use std::collections::BTreeMap;
use std::error::Error;
use std::io::prelude::*;
use std::{env, fs};

fn main() -> Result<(), Box<dyn Error>> {
    let args: Vec<String> = env::args().collect();
    let file = fs::File::open(
        args.get(1)
            .expect("No file supplied in command line invocation"),
    )?;
    let peer_id = make_peer_id();
    let root_dict = read_torrent(file)?;
    let info_bencoded = bencode_info(&root_dict);
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
        peer.write_to_peer(handshake.as_slice())?;
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

fn bencode_info(root_dict: &BTreeMap<Vec<u8>, Bencode>) -> Vec<u8> {
    let info_bencoded = root_dict
        .get("info".as_bytes())
        .expect("No 'info' key in torrent file")
        .encode_val();
    info_bencoded
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

// Read .torrent file and de-bencode it, First value in a .torrent should be a bencoded dictionary.
fn read_torrent(mut file: fs::File) -> Result<BTreeMap<Vec<u8>, Bencode>, std::io::Error> {
    let mut buf = Vec::with_capacity(1_000_000);
    let _bytes_read = file.read_to_end(&mut buf);
    let root_dict = match Bencode::decode_dispatch(&mut buf.iter().peekable())? {
        Bencode::Dict(d) => d,
        _ => panic!("Top level bencoded value is not a dictionary"),
    };
    Ok(root_dict)
}

// Generates a peer id in the Azureus-style described here: https://wiki.theory.org/BitTorrentSpecification#peer_id
// Chose "AI" as the client tag because I didn't see it in use.
fn make_peer_id() -> String {
    const PEER_TAG: [u8; 8] = *b"-AI0001-";
    use rand::Rng;
    let mut res = Vec::new();
    res.extend(PEER_TAG);
    let mut rng = rand::thread_rng();
    for _ in 0..12 {
        res.push(rng.gen_range(b'0'..=b'9'));
    }
    String::from_utf8(res).unwrap()
}

// Construct handshake byte string to send to peers
//len(info_hash) + len(peer_id) + 8 reserved bytes + 1 (byte declaring length of protocol string)
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
