#![allow(dead_code)]

use core::num;
use std::collections::BTreeMap;

use bencode::Bencode;

pub mod bencode;
pub mod decode;
pub mod file_dict;

use file_dict::FileDict;
use url::form_urlencoded::parse;
// use reqwest::RequestBuilder;
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
    pub fn construct_from_dict_v1(root_dict: BTreeMap<ByteString, Bencode>) -> MetaInfo {
        MetaInfo {
            announce: Self::get_message(&root_dict, "announce".as_bytes()).unwrap(),
            // TODO: Implement announce-list
            creation_date: Self::get_int(&root_dict, "creation date".as_bytes()),
            comment: Self::get_message(&root_dict, "comment".as_bytes()),
            created_by: Self::get_message(&root_dict, "created by".as_bytes()),
            encoding: Self::get_message(&root_dict, "encoding".as_bytes()),
            url_list: Self::get_url_list(&root_dict),
            info: FileDict::construct_from_info(root_dict.get("info".as_bytes()).unwrap()),
            announce_list: None,
        }
    }

    fn get_message(d: &BTreeMap<ByteString, Bencode>, key: &[u8]) -> Option<ByteString> {
        d.get(key).map(|b| b.unwrap_message())
    }

    fn get_int(d: &BTreeMap<ByteString, Bencode>, key: &[u8]) -> Option<isize> {
        d.get(key).map(|b| b.unwrap_int())
    }
    fn get_url_list(d: &BTreeMap<ByteString, Bencode>) -> Option<Vec<ByteString>> {
        d.get("url-list".as_bytes()).map(|b| {
            b.unwrap_list()
                .iter()
                .map(|url| url.unwrap_message())
                .collect()
        })
    }
}

pub struct GetRequest {
    pub compact: u8,
    pub downloaded: String,
    pub event: Option<String>,
    pub info_hash: ByteString,
    pub ip: Option<String>,
    pub key: Option<ByteString>,
    pub left: String,
    pub pubnumwant: Option<usize>,
    pub peer_id: ByteString,
    pub port: String,
    pub trackerid: Option<ByteString>,
    pub uploaded: String,
}

// impl GetRequest {
//     pub fn make_get_reqwest_body(&self) -> RequestBuilder {
//         RequestBuilder::body(self, body);
//     }
// }

pub struct TrackerResponse {
    pub warning_reason: Option<String>,
    pub interval: usize,
    pub min_interval: Option<usize>,
    pub tracker_id: Option<ByteString>,
    pub complete: isize,
    pub incomplete: isize,
}

// impl TrackerResponse {
//     pub fn parse_response_string(response:String) -> TrackerResponse {
//         let dict = Bencode::decode_all(response.as_bytes()).get(0).unwrap();
//         TrackerResponse { warning_reason: None, interval: 900, min_interval: None, tracker_id: None, complete: 1, incomplete: 1 }
//     }
// }

use std::net::{SocketAddrV4,Ipv4Addr};
pub fn extract_peers_from_compact_response(bytes: Vec<u8>) -> Vec<SocketAddrV4> {
    if bytes.len() % 6 != 0 {
        panic!("Comapct peers byte string is not a multiple of 6. Impossible to parse");
    }
    let num_addrs = bytes.len() / 6;
    let mut parsed_peers = Vec::<SocketAddrV4>::with_capacity(num_addrs);
    let mut it = bytes.into_iter();
    for _ in 0..num_addrs {
        let x:Vec<u8> = it.by_ref().take(6).collect();
        let port = u16::from_be_bytes([x[4],x[5]]);
        parsed_peers.push(SocketAddrV4::new(Ipv4Addr::new(x[0], x[1], x[2], x[3]), port));
    }
    parsed_peers
}
