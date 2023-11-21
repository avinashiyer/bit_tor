use anyhow::anyhow;
use bittorrent_starter_rust::{build_tracker_url, escape_u8_slice, print_value, Metainfo, build_handshake};
use serde_bencode::value::Value;
use std::{
    env,
    fs::File,
    io::{BufReader, Read},
    net::{Ipv4Addr, SocketAddrV4},
};
#[tokio::main]
async fn main() -> Result<(), anyhow::Error> {
    let args: Vec<String> = env::args().collect();
    let command = &args[1];
    let target = &args[2];
    if command == "decode" {
        let decoded_value: Value = serde_bencode::from_str(target).expect("Invalid Data");
        print_value(&decoded_value);
        println!();
    } else if command == "info" {
        let metainfo = get_torrent_info(target);
        print_metainfo(metainfo);
    } else if command == "peers" {
        let metainfo = get_torrent_info(target);
        let peers = get_peers(&metainfo).await?;
        peers.iter().for_each(|sock| println!("{}", sock));
    } else if command == "handshake" {
        let metainfo = get_torrent_info(target);
        let handshake = build_handshake(&metainfo);
        let peers = get_peers(&metainfo).await?;
        

    } else {
        println!("unknown command: {}", args[1])
    }
    Ok(())
}

fn get_torrent_info(file_path: &String) -> Metainfo {
    let file = File::open(file_path).expect("No file with specified path");
    let mut file_reader = BufReader::new(file);
    let mut bytes_buf: Vec<u8> = Vec::with_capacity(2 ^ 16);
    file_reader
        .read_to_end(&mut bytes_buf)
        .expect("Error reading torrent file");
    let decoded_torrent_file: Value =
        serde_bencode::from_bytes(&bytes_buf).expect("Error decoding torrent");
    Metainfo::new(&decoded_torrent_file)
}

fn print_metainfo(metainfo: Metainfo) {
    let hex_string = hex::encode(metainfo.hashed_info);
    let pieces_vec: Vec<&[u8]> = metainfo.pieces.chunks_exact(20).collect();
    let pieces_strings_vec = pieces_vec.iter().map(hex::encode);
    println!("Tracker URL: {}", escape_u8_slice(&metainfo.announce));
    println!("Length: {}", metainfo.length);
    println!("Info Hash: {}", hex_string);
    println!("Piece Length: {}", metainfo.piece_length);
    println!("Piece Hashes:");
    pieces_strings_vec.for_each(|s| println!("{s}"));
}

async fn get_peers(metainfo: &Metainfo) -> Result<Vec<SocketAddrV4>, anyhow::Error> {
    let request_str = build_tracker_url(metainfo, 0, 0, metainfo.length as usize)?;
    let response = reqwest::get(request_str)
        .await?
        .error_for_status()?
        .bytes()
        .await?;
    let response_dict = match serde_bencode::from_bytes(&response)? {
        Value::Dict(d) => d,
        _ => {
            return Err(anyhow!("Response is not a bencoded dictionary"));
        }
    };
    match response_dict.get("peers".as_bytes()) {
        Some(Value::Bytes(b)) => Ok(parse_peers(b.to_vec())?),
        Some(_) => Err(anyhow!("Peers is not a list")),
        None => Err(anyhow!("No peers key in dictionary")),
    }
}

fn parse_peers(peer_bytes: Vec<u8>) -> Result<Vec<SocketAddrV4>, anyhow::Error> {
    let mut res = Vec::<SocketAddrV4>::new();
    assert!(peer_bytes.len() % 6 == 0);
    for octets in peer_bytes.chunks_exact(6) {
        res.push(peer_helper(octets));
    }
    Ok(res)
}

fn peer_helper(bytes: &[u8]) -> SocketAddrV4 {
    assert!(bytes.len() == 6);
    let port: u16 = u16::from_be_bytes([bytes[4], bytes[5]]);
    let octets: [u8; 4] = bytes[..4]
        .try_into()
        .expect("Could not convert byte slice into sized array");
    SocketAddrV4::new(Ipv4Addr::from(octets), port)
}
