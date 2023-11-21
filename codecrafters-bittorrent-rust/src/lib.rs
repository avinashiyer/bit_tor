

use anyhow::Error;
use serde_bencode::value::Value;
use sha1::{Digest, Sha1};

pub const PEER_ID: &str = "AV000115489562354851";

pub fn print_value(val: &serde_bencode::value::Value) {
    match val {
        Value::Bytes(vec) => print!("\"{}\"", escape_u8_slice(vec)),
        Value::Int(num) => print!("{num}"),
        Value::List(ls) => {
            print!("[");
            if let Some((last, elems)) = ls.split_last() {
                elems.iter().for_each(|item| {
                    print_value(item);
                    print!(",")
                });
                print_value(last);
            }
            print!("]")
        }
        Value::Dict(map) => {
            print!("{{");
            let mut entries = map.iter().collect::<Vec<(&Vec<u8>, &Value)>>();
            entries.sort_by_key(|tuple| tuple.0);
            if let Some((last, elems)) = entries.split_last() {
                elems.iter().for_each(|(key, val)| {
                    print!("\"{}\":", escape_u8_slice(key));
                    print_value(val);
                    print!(",");
                });
                print!("\"{}\":", escape_u8_slice(last.0));
                print_value(last.1);
            }
            print!("}}");
        }
    }
}

pub struct Metainfo {
    pub announce: Vec<u8>,
    pub length: i64,
    pub name: Vec<u8>,
    pub piece_length: i64,
    pub pieces: Vec<u8>,
    pub hashed_info: Vec<u8>,
}

impl Metainfo {
    pub fn new(serde_map: &Value) -> Metainfo {
        if let Value::Dict(root_dict) = serde_map {
            let Some(info_val) = root_dict.get(str::as_bytes("info")) else {
                panic!("Missing metainfo (info)");
            };
            let bencoded_info = serde_bencode::to_bytes(info_val).expect("Error bencoding info");
            let mut hasher = Sha1::new();
            hasher.update(bencoded_info);
            let hashed_info: Vec<u8> = hasher.finalize().into_iter().collect();
            let Some(Value::Bytes(announce)) = root_dict.get(str::as_bytes("announce")) else {
                panic!("No tracker url in torrent file");
            };
            let Value::Dict(info_dict) = info_val else {
                panic!("info does not map to a dicitonary ");
            };
            let Some(Value::Int(length)) = info_dict.get(str::as_bytes("length")) else {
                panic!("Missing metainfo (length)");
            };
            let Some(Value::Bytes(name)) = info_dict.get(str::as_bytes("name")) else {
                panic!("Missing metainfo (name)");
            };
            let Some(Value::Int(piece_length)) = info_dict.get(str::as_bytes("piece length"))
            else {
                panic!("missing metainfoinfo (piece length)");
            };
            let Some(Value::Bytes(pieces)) = info_dict.get(str::as_bytes("pieces")) else {
                panic!("missing metainfo (pieces)");
            };
            return Metainfo {
                announce: (announce.clone()),
                length: (*length),
                name: (name.clone()),
                piece_length: (*piece_length),
                pieces: (pieces.clone()),
                hashed_info,
            };
        }
        panic!();
    }
}

pub fn build_handshake(meta: &Metainfo) -> Vec<u8> {
    let mut res = Vec::new();
    res.extend("19:BitTorrent protocol".as_bytes().iter());
    res.extend([0].iter().cycle().take(8));
    res.extend(meta.hashed_info.iter());
    res.extend(PEER_ID.as_bytes().iter());
    res
}

pub fn build_tracker_url(
    meta: &Metainfo,
    uploaded: usize,
    downloaded: usize,
    left: usize,
) -> Result<String, Error> {
    let info_hash_urlcoded: String = percent_encode(&meta.hashed_info)?;
    let url_encoded_str = serde_urlencoded::to_string([
        ("peer_id", PEER_ID.to_owned()),
        ("port", "6881".to_owned()),
        ("uploaded", uploaded.to_string()),
        ("downloaded", downloaded.to_string()),
        ("left", left.to_string()),
        ("compact", "1".to_owned()),
    ])?;
    Ok(format!("{}?info_hash={info_hash_urlcoded}&{url_encoded_str}",escape_u8_slice(&meta.announce)))
}

fn percent_encode(src: &[u8]) -> Result<String, anyhow::Error> {
    let reserved_chars: Vec<u8> = vec![
        b' ', b'!', b'"', b'#', b'$', b'%', b'&', b'\'', b'(', b')', b'*', b'+', b',', b'/', b':',
        b';', b'=', b'?', b'@', b'[', b']',
    ];
    let mut res = Vec::<u8>::new();
    for byte in src {
        if byte.is_ascii_alphanumeric() && !reserved_chars.contains(byte) {
            res.push(*byte);
        } else {
            res.push(b'%');
            res.extend(format!("{:02x}", byte).as_bytes());
        }
    }
    Ok(String::from_utf8(res)?)
}

//Helper method that '\' ecapes whitespaces and '\xx' escapes other non-printables
pub fn escape_u8_slice(src: &[u8]) -> String {
    String::from_utf8(
        src.iter()
            .flat_map(|b| std::ascii::escape_default(*b))
            .collect::<Vec<u8>>(),
    )
    .unwrap()
}
