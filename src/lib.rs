#![allow(dead_code)]

use std::collections::BTreeMap;
use std::io::{BufRead, BufReader, Write, Read};
use std::net::{Ipv4Addr, SocketAddrV4, TcpStream};

use percent_encoding::{percent_encode, AsciiSet, NON_ALPHANUMERIC};

use bencode::Bencode;
use file_dict::FileDict;

pub mod bencode;
pub mod decode;
pub mod file_dict;

const ESCAPED_CHARACTERS: &AsciiSet = &NON_ALPHANUMERIC
    .remove(b'.')
    .remove(b'-')
    .remove(b'_')
    .remove(b'~');

pub struct MetaInfo {
    pub announce: Vec<u8>,
    pub announce_list: Option<Vec<Vec<u8>>>,
    pub creation_date: Option<isize>,
    pub comment: Option<Vec<u8>>,
    pub created_by: Option<Vec<u8>>,
    pub encoding: Option<Vec<u8>>,
    pub url_list: Option<Vec<Vec<u8>>>,
    pub info: FileDict,
    pub info_hash: [u8; 20],
    pub escaped_hash: String,
}
impl MetaInfo {
    pub fn construct_from_dict_v1(
        root_dict: BTreeMap<Vec<u8>, Bencode>,
        hashed_info: [u8; 20],
    ) -> MetaInfo {
        let escaped_hash = percent_encode(&hashed_info, ESCAPED_CHARACTERS).to_string();
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
            info_hash: hashed_info,
            escaped_hash,
        }
    }

    pub fn tracker_get(meta_info: &MetaInfo, peer_id: String) -> Result<Vec<u8>,reqwest::Error> {
        let announce_url_utf8 = std::str::from_utf8(&meta_info.announce)
            .expect("Error converting announce url to utf-8 encoding");
        // let bytes_left = meta.info.file_length.unwrap().to_string();
        let res = format!(
            "{announce_url_utf8}?\
            info_hash={escaped_hash}&\
            event=started&\
            peer_id={peer_id}\
            &compact=1\
            &numwant=5",
            escaped_hash = meta_info.escaped_hash
        );
        Ok(reqwest::blocking::get(res)?.bytes().unwrap().iter().copied().collect())
    }

    fn get_message(d: &BTreeMap<Vec<u8>, Bencode>, key: &[u8]) -> Option<Vec<u8>> {
        d.get(key).map(|b| b.unwrap_message())
    }

    fn get_int(d: &BTreeMap<Vec<u8>, Bencode>, key: &[u8]) -> Option<isize> {
        d.get(key).map(|b| b.unwrap_int())
    }

    fn get_url_list(d: &BTreeMap<Vec<u8>, Bencode>) -> Option<Vec<Vec<u8>>> {
        d.get("url-list".as_bytes()).map(|b| {
            b.unwrap_list()
                .iter()
                .map(|url| url.unwrap_message())
                .collect()
        })
    }
}

#[derive(Debug)]
pub struct Peer {
    pub am_choking: u8,
    pub am_interested: u8,
    pub peer_choking: u8,
    pub peer_interested: u8,
    pub socket: SocketAddrV4,
    pub handle: TcpStream,
    pub buffer: Vec<u8>,
}

impl Peer {
    pub fn new_peer(socket: SocketAddrV4) -> Option<Peer> {
        let handle = match TcpStream::connect_timeout(
            &std::net::SocketAddr::V4(socket),
            std::time::Duration::new(5, 0),
        ) {
            Ok(h) => h,
            Err(_) => return None,
        };
        Some(Peer {
            am_choking: 1,
            am_interested: 0,
            peer_choking: 1,
            peer_interested: 0,
            socket,
            handle,
            buffer: Vec::<u8>::new(),
        })
    }

    pub fn get_peers(response: Vec<u8>) -> Result<Vec<Peer>,std::io::Error> {
        let mut response_iter = response.iter().peekable();
        let bencoded_response = Bencode::decode_dispatch(&mut response_iter)?;
        let tracker_response_dict = bencoded_response.unwrap_dict();
        if let Some(x) = tracker_response_dict.get("failure reason".as_bytes()) {
            return Err(make_bad_data_err(
                &format!("Tracker Request Failed with reason: \n{}",
                escape_u8_slice(&x.unwrap_message()))
            ));
        }
        match tracker_response_dict.get("peers".as_bytes()) {
            None => Err(make_bad_data_err("No peers in tracker respone")),
            Some(Bencode::Dict(_)) => {
                Err(make_bad_data_err("Non Compact response from tracker recieved"))
            }
            Some(Bencode::Message(m)) => Peer::extract_peers_from_compact_response(m.to_vec()),
            _ => Err(make_bad_data_err("Peers encoded in non recognized format")),
        }
    }

    pub fn extract_peers_from_compact_response(bytes: Vec<u8>) -> Result<Vec<Peer>,std::io::Error> {
        if bytes.len() % 6 != 0 {
            return Err(make_bad_data_err("Comapct peers byte string is not a multiple of 6. Impossible to parse"));
        }
        let num_addrs = bytes.len() / 6;
        let mut parsed_peers = Vec::<SocketAddrV4>::with_capacity(num_addrs);
        let mut it = bytes.into_iter();
        for _ in 0..num_addrs {
            let x: Vec<u8> = it.by_ref().take(6).collect();
            let port = u16::from_be_bytes([x[4], x[5]]);
            parsed_peers.push(SocketAddrV4::new(
                Ipv4Addr::new(x[0], x[1], x[2], x[3]),
                port,
            ));
        }
        Ok(parsed_peers
            .into_iter()
            .filter_map(Peer::new_peer)
            .collect())
    }

    pub fn write_to_peer(&mut self, message: &[u8]) -> std::io::Result<()>{
        self.handle.write_all(message)?;
        self.handle.flush()
    }

    pub fn read_peer_message(&mut self) -> std::io::Result<()> {
        let mut length_prefix = [0u8;4];
        self.handle.read_exact(&mut length_prefix)?;
        let length = u32::from_be_bytes(length_prefix);
        if length == 0 {
            return Ok(());
        }
        Ok(())
    }
}

pub fn make_bad_data_err(err_msg: &str) -> std::io::Error {
    std::io::Error::new(std::io::ErrorKind::InvalidData, err_msg)
}


pub fn escape_u8_slice(src: &[u8]) -> String {
    String::from_utf8(
        src.iter()
            .flat_map(|b| std::ascii::escape_default(*b))
            .collect::<Vec<u8>>(),
    )
    .unwrap()
}


pub enum Message{

} 
