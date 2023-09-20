#![allow(dead_code)]

use std::collections::BTreeMap;

use bencode::Bencode;

pub mod bencode;
pub mod decode;
pub mod file_dict;

use file_dict::FileDict;
type ByteString = Vec<u8>;

pub struct MetaInfo {
    pub announce: ByteString,
    pub announce_list: Option<Vec<ByteString>>,
    pub creation_date: Option<isize>,
    pub comment: Option<ByteString>,
    pub created_by: Option<ByteString>,
    pub encoding: Option<ByteString>,
    pub url_list: Option<Vec<ByteString>>,
    pub info: FileDict,
}
impl MetaInfo {
    pub fn construct_from_dict_v1(root_dict: Bencode) -> MetaInfo {
        let d: BTreeMap<ByteString, Bencode> = match root_dict {
            Bencode::Dict(d) => d,
            _ => panic!("Bencode passed in is not a dictionary"),
        };
        MetaInfo {
            announce : Self::get_message(&d, "announce".as_bytes()).unwrap(),
            // TODO: Implement announce-list
            creation_date : Self::get_int(&d, "creation date".as_bytes()),
            comment : Self::get_message(&d, "comment".as_bytes()),
            created_by : Self::get_message(&d, "created by".as_bytes()),
            encoding : Self::get_message(&d, "encoding".as_bytes()),
            url_list : Self::get_url_list(&d),
            info : FileDict::construct_from_info(d.get("info".as_bytes()).unwrap()),
            announce_list : None,
        }

    }

    fn get_message(d: &BTreeMap<ByteString, Bencode>, key: &[u8]) -> Option<ByteString> {
        d.get(key).and_then(|b| Some(b.unwrap_message()))
    }

    fn get_int(d: &BTreeMap<ByteString, Bencode>, key: &[u8]) -> Option<isize> {
        d.get(key).and_then(|b| Some(b.unwrap_int()))
    }
    fn get_url_list(d: &BTreeMap<ByteString, Bencode>) -> Option<Vec<ByteString>> {
        d.get("url-list".as_bytes()).and_then(|b| {
            Some(
                b.unwrap_list()
                    .iter()
                    .map(|url| url.unwrap_message())
                    .collect(),
            )
        })
    }
}


pub struct GetRequest {
    info_hash: ByteString,
    peer_id: ByteString,
    port: String,
    uploaded: String,
    downloaded: String,
    left: String,
    compact: u8,
    event: Option<String>,
    ip: Option<String>,
    numwant: Option<usize>,
    key: Option<ByteString>,
    trackerid: Option<ByteString>,
}

pub struct TrackerResponse {
    warning_reason: Option<String>,
    interval: usize,
    min_interval: Option<usize>,
    tracker_id: Option<ByteString>,
    complete: isize,
    incomplete: isize,
}
