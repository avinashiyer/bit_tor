#![allow(dead_code)]

use std::collections::BTreeMap;

use bencode::Bencode;

pub mod bencode;
pub mod decode;
pub mod file_dict;

use file_dict::FileDict;
use reqwest::RequestBuilder;
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
use reqwest;

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
