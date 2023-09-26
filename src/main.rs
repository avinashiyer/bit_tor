use bit_tor::bencode::Bencode;
use bit_tor::{GetRequest, MetaInfo};
use core::panic;
use percent_encoding::{percent_encode, AsciiSet, NON_ALPHANUMERIC};
use reqwest;
use std::collections::BTreeMap;
use std::fs;
use std::io::prelude::*;
use std::net::IpAddr;

// use std::thread;
// use std::time::Duration;
const ESCAPED_CHARACTERS: &AsciiSet = &NON_ALPHANUMERIC
    .remove(b'.')
    .remove(b'-')
    .remove(b'_')
    .remove(b'~');

fn main() -> std::io::Result<()> {
    let mut file = fs::File::open(
        "/home/avi/rust_prac/bit_tor/src/debian-edu-12.1.0-amd64-netinst.iso.torrent",
    )
    .unwrap();
    let mut buf = Vec::with_capacity(1_000_000);
    let _bytes_read = file.read_to_end(&mut buf);
    let root_dict = match Bencode::decode_single(&mut buf.iter().peekable()) {
        Bencode::Dict(d) => d,
        _ => panic!("Top level bencoded value is not a dictionary"),
    };
    let info_bencoded = root_dict
        .get("info".as_bytes())
        .expect("No info value")
        .encode_val();
    let info_hash = sha1_smol::Sha1::from(info_bencoded).digest().bytes();
    let escaped_info_hash = percent_encode(&info_hash, ESCAPED_CHARACTERS).to_string();
    let meta_info = MetaInfo::construct_from_dict_v1(root_dict);
    fun_name(meta_info, escaped_info_hash);
    Ok(())
}

fn fun_name(meta_info: MetaInfo, escaped_info_hash: String) {
    let req = reqwest::blocking::get(make_get_request(&meta_info, escaped_info_hash))
        .unwrap()
        .text()
        .unwrap();
    println!("{req}");
}

fn _get_tracker_url(src: &BTreeMap<Vec<u8>, Bencode>) -> String {
    if let Some(url_bencode) = src.get("announce".as_bytes()) {
        if let Bencode::Message(url) = url_bencode {
            return String::from_utf8(url.clone()).expect("Invalid Url in announce field");
        }
        panic!("Value under \"announce\" key is not a bencoded string.")
    } else {
        panic!("Ill formatted torrent file, no \"announce\" key in root dictionary.");
    }
}

fn make_peer_id() -> String {
    use rand::Rng;
    let mut res = vec![b'-', b'A', b'V', b'0', b'0', b'0', b'1', b'-'];
    let mut rng = rand::thread_rng();
    for _ in 0..12 {
        res.push(rng.gen_range(b'0'..b'9'));
    }
    String::from_utf8(res).unwrap()
}

fn make_get_request(meta: &MetaInfo, escaped_hash: String) -> String {
    let announce_url_utf8 = std::str::from_utf8(&meta.announce)
        .expect("Error converting announce url to utf-8 encoding");
    let peer_id = make_peer_id();
    let bytes_left = meta.info.file_length.unwrap().to_string();
    let res = format!(
        "{announce_url_utf8}?info_hash={escaped_hash}&peer_id={peer_id}&port=6881&uploaded=0&downloaded=0&left={bytes_left}&compact=1&event=started&");
    res.to_string()
}
