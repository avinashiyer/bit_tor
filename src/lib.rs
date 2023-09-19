#![allow(dead_code)]


pub mod bencode;
pub mod decode;

pub struct MetaInfo {
    announce: Vec<u8>,
    announce_list: Option<Vec<Vec<u8>>>,
    creation_date: Option<isize>,
    comment: Option<Vec<u8>>,
    created_by: Option<Vec<u8>>,
    encoding: Option<Vec<u8>>,
    info: FileDict,
}

pub struct FileDict {
    piece_length:isize,
    pieces:Vec<u8>,
    single_file:bool,
    multi:Option<MultiFileInfo>,
    single:Option<SingleFileInfo>,

}

struct FileInfo{
    length:isize,
    path:Option<Vec<u8>>,
}

pub struct MultiFileInfo {
    name: Vec<u8>,
    files:Vec<FileInfo>

}

pub struct  SingleFileInfo {
    name: Vec<u8>,
    length: isize,
}
