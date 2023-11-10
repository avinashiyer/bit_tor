use std::collections::BTreeMap;

use crate::bencode::Bencode;
type ByteString = Vec<u8>;

pub struct FileDict {
    pub piece_length: isize,
    pub pieces: Vec<ByteString>,
    pub single_file: bool,
    pub files: Option<Vec<FileInfo>>,
    pub name: ByteString,
    pub file_length: Option<isize>,
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
        let piece_hashes: Vec<ByteString> = pieces
            .chunks_exact(20)
            .map(|chunk| chunk.to_vec())
            .collect();
        let mut file_length: Option<isize> = None;
        let file_name: ByteString;
        let mut file_list: Option<Vec<FileInfo>> = None;
        let single_file = match FileOrDir::from_dict(info_dict) {
            FileOrDir::Single(SingleFileInfo { name, length }) => {
                file_length = Some(length);
                file_name = name;
                true
            }
            FileOrDir::Multi(MultiFileInfo { dir_name, files }) => {
                file_list = Some(files);
                file_name = dir_name;
                false
            }
        };
        FileDict {
            piece_length: piece_len,
            pieces: piece_hashes,
            single_file,
            file_length,
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
    pub fn from_dict(dict: &BTreeMap<ByteString, Bencode>) -> FileOrDir {
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
            FileOrDir::Single(s_file_info)
        }
    }
}

pub struct FileInfo {
    length: isize,
    path: Vec<ByteString>,
}

pub struct MultiFileInfo {
    dir_name: ByteString,
    files: Vec<FileInfo>,
}

pub struct SingleFileInfo {
    name: ByteString,
    length: isize,
}
