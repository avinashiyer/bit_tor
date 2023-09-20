#![allow(dead_code)]

use std::{collections::BTreeMap, fs::File};

use bencode::Bencode;

pub mod bencode;
pub mod decode;
type byte_string = Vec<u8>;

pub struct MetaInfo {
    announce: byte_string,
    announce_list: Option<Vec<byte_string>>,
    creation_date: Option<isize>,
    comment: Option<byte_string>,
    created_by: Option<byte_string>,
    encoding: Option<byte_string>,
    url_list: Option<Vec<byte_string>>,
    info: FileDict,
}
impl MetaInfo {
    pub fn construct_from_dict_v1(root_dict: Bencode) -> MetaInfo {
        let mut d: BTreeMap<byte_string, Bencode> = match root_dict {
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

    fn get_message(d: &BTreeMap<byte_string, Bencode>, key: &[u8]) -> Option<byte_string> {
        d.get(key).and_then(|b| Some(b.unwrap_message()))
    }

    fn get_int(d: &BTreeMap<byte_string, Bencode>, key: &[u8]) -> Option<isize> {
        d.get(key).and_then(|b| Some(b.unwrap_int()))
    }
    fn get_url_list(d: &BTreeMap<byte_string, Bencode>) -> Option<Vec<byte_string>> {
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

pub struct FileDict {
    piece_length: isize,
    pieces: Vec<byte_string>,
    single_file: bool,
    files: Option<Vec<FileInfo>>,
    name: byte_string,
    file_length: Option<isize>,
}

impl FileDict {
    pub fn construct_from_info(bencode_dict: &Bencode) -> FileDict {
        let info_dict = bencode_dict.unwrap_dict();
        let piece_len = info_dict
            .get("piece length".as_bytes())
            .expect("No length value in info dict")
            .unwrap_int();
        let pieces = info_dict
            .get("pieces".as_bytes())
            .expect("No pieces value in info dict")
            .unwrap_message();
        if pieces.len() % 20 != 0 {
            panic!("pieces is not a multiple of 20");
        }
        let piece_hashes: Vec<byte_string> = pieces
            .chunks_exact(20)
            .map(|chunk| chunk.to_vec())
            .collect();
        let mut file_length: Option<isize> = None;
        let file_name: byte_string;
        let mut file_list: Option<Vec<FileInfo>> = None;
        let single_file: bool;
        match FileOrDir::from_dict(info_dict) {
            FileOrDir::Single(SingleFileInfo { name, length }) => {
                file_length = Some(length);
                file_name = name;
                single_file = true
            }
            FileOrDir::Multi(MultiFileInfo { dir_name, files }) => {
                file_list = Some(files);
                file_name = dir_name;
                single_file = false
            }
        }
        FileDict {
            piece_length: piece_len,
            pieces: piece_hashes,
            single_file: single_file,
            file_length: file_length,
            name: file_name,
            files: file_list,
        }
    }
}

enum FileOrDir {
    Single(SingleFileInfo),
    Multi(MultiFileInfo),
}

impl FileOrDir {
    fn extract_file_info(file_info_bencoded: &Vec<Bencode>) -> Vec<FileInfo> {
        let mut file_info_extracted: Vec<FileInfo> = Vec::with_capacity(file_info_bencoded.len());
        for ben_val in file_info_bencoded {
            let d = ben_val.unwrap_dict();
            file_info_extracted.push(FileInfo {
                length: d
                    .get("length".as_bytes())
                    .expect("Bad length in files")
                    .unwrap_int(),
                path: d
                    .get("path".as_bytes())
                    .expect("Bad path in files")
                    .unwrap_list()
                    .iter()
                    .map(|v| v.unwrap_message())
                    .collect(),
            })
        }
        file_info_extracted
    }
    pub fn from_dict(dict: &BTreeMap<byte_string, Bencode>) -> FileOrDir {
        if dict.contains_key("files".as_bytes()) {
            let file_info_bencoded: Vec<Bencode> = match dict.get("files".as_bytes()).unwrap() {
                Bencode::List(l) => l.to_vec(),
                _ => panic!("files maps to a non bencoded list"),
            };
            let file_info_list = Self::extract_file_info(&file_info_bencoded);
            return FileOrDir::Multi(MultiFileInfo {
                dir_name: dict
                    .get("name".as_bytes())
                    .expect("No name key")
                    .unwrap_message(),
                files: file_info_list,
            });
        } else {
            let s_file_info = SingleFileInfo {
                name: dict
                    .get("name".as_bytes())
                    .expect("No name key")
                    .unwrap_message(),
                length: dict
                    .get("length".as_bytes())
                    .expect("No length key")
                    .unwrap_int(),
            };
            return FileOrDir::Single(s_file_info);
        }
    }
}

pub struct FileInfo {
    length: isize,
    path: Vec<byte_string>,
}

pub struct MultiFileInfo {
    dir_name: byte_string,
    files: Vec<FileInfo>,
}

pub struct SingleFileInfo {
    name: byte_string,
    length: isize,
}

pub struct GetRequest {
    info_hash: byte_string,
    peer_id: byte_string,
    port: String,
    uploaded: String,
    downloaded: String,
    left: String,
    compact: u8,
    event: Option<String>,
    ip: Option<String>,
    numwant: Option<usize>,
    key: Option<byte_string>,
    trackerid: Option<byte_string>,
}

pub struct TrackerResponse {
    warning_reason: Option<String>,
    interval: usize,
    min_interval: Option<usize>,
    tracker_id: Option<byte_string>,
    complete: isize,
    incomplete: isize,
}
