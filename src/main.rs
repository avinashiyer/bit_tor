use bit_tor::bencode::Bencode;
use bit_tor::MetaInfo;
use core::panic;
use std::collections::BTreeMap;
use std::fs;
use std::io::prelude::*;
use std::net::TcpStream;

use url::Url;
// use std::thread;
// use std::time::Duration;

fn main() -> std::io::Result<()> {
    let mut file = fs::File::open(
        "/home/avi/rust_prac/bit_tor/src/debian-edu-12.1.0-amd64-netinst.iso.torrent",
    )
    .unwrap();
    let mut buf = Vec::with_capacity(1_000_000);
    let _bytes_read = file.read_to_end(&mut buf);
    let meta_info =
        MetaInfo::construct_from_dict_v1(Bencode::decode_single(&mut buf.iter().peekable()));
    let url = meta_info.announce;
    
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
