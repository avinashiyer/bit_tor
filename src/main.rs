use bit_tor::bencode::Bencode;
use bit_tor::{GetRequest, MetaInfo};
use core::panic;
use percent_encoding::{percent_encode, AsciiSet, NON_ALPHANUMERIC};
use reqwest;
use std::collections::BTreeMap;
use std::fs;
use std::io::prelude::*;

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
    let escaped_info_hash = percent_encode(&info_hash, ESCAPED_CHARACTERS);
    let meta_info = MetaInfo::construct_from_dict_v1(root_dict);
    let utf_8_str = std::str::from_utf8(&meta_info.announce)
        .expect("Error converting announce url to utf-8 encoding");
    // let
    // let s = GetRequest {
    //     compact:0,
    //     downloaded:String::from("0"),
    //     event: Some(String::from("started")),
    //     info_hash:

    //}

    Ok(())
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
