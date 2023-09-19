#![allow(dead_code)]
use std::collections::HashMap;

use bencode::Bencode;

pub mod bencode;
pub mod decode;

// pub struct MetaInfo {
//     announce: Vec<u8>,
//     announce_list: Option<Vec<Vec<u8>>>,
//     creation_date: Option<isize>,
//     comment: String,
//     created_by: String,
//     encoding: String,
//     info: 
// }

// pub struct FileDict {
//     piece_length:isize,
//     pieces:Vec<u8>,
    
// }

// struct FileInfo{
//     length:isize,
//     path:Option<Bencode>,
// }

// pub struct MultiFileInfo {
    

//     name: Vec<u8>,
//     files:Vec<
// }
