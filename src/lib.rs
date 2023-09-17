#![allow(dead_code)]
use std::collections::HashMap;

pub mod bencode;
pub mod decode;

struct InfoD {
    name : Option<String>,
    piece_len :isize,
    meta_vers : u8,
    file_tree : FileTree,
    length : Option<isize>,
    pieces_root : String,


}

struct Metainfo {
    announce : String,
    info : FileTree,
    piece_layers : HashMap<String,String>,
}

enum FileTree {
    File(String),
    HMap(HashMap<FileTree,FileTree>),
}

struct GetRequest {
    info_hash:String,
    peer_id:String,
    ip:Option<String>,
    port:u16,
    uploaded:isize,
    downloaded:isize,
    left:isize,
    event:Option<String>,
}