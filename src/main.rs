use bit_tor::bencode::Bencode;
use core::{panic, num};
use std::collections::BTreeMap;
use std::fs;
use std::io::prelude::*;
use std::net::UdpSocket;
use url::Url;



// use std::thread;
// use std::time::Duration;

fn main() -> std::io::Result<()>{
    let file = fs::File::open("/home/avi/rust_prac/bit_tor/src/debian.iso.torrent").unwrap();
    let f_iter = file.bytes()
    // let torrent_dict: &BTreeMap<Vec<u8>, Bencode>;
    // let a = f_iter.count();
    Bencode::decode_all(f_iter);
    // println!("{}",helper(f_iter));
    
    // if let Some(Bencode::Dict(dict)) = torrent_vec.get(0) {
    //     torrent_dict = dict;
    // } else {
    //     panic!("Torrent file is not a bencoded dictionary");
    // }
    // let announce_list = match torrent_dict.get("announce-list".as_bytes()).unwrap() {
    //     Bencode::List(l) => l,
    //     _ => panic!("Bad bencode under \"announce-list\" key in root dictionary)"),
    // };
    // let announce_list = flatten_announce_list(announce_list);
    // let url = Url::parse(&announce_list[0]).unwrap();
    // let addrs = url.socket_addrs(|| None).unwrap();
    // println!("{addrs:#?}");
    // let socket = UdpSocket::bind("127.0.0.1:3400").expect("Couldnt Bind");
    // socket.connect("199.59.243.224:6969").expect("Couldn't connect to remote");
    // let mut buf = Vec::<u8>::new();
    // let (number_of_bytes, src_addr) = socket.recv_from(&mut buf)
    //                                     .expect("Didn't receive data");
    // println!("{:#?}",buf);




    Ok(())
}

pub fn helper(v:Vec<u8>) -> String{
    let mut r:Vec<String> = Vec::new();
    for c in v {
        match c {
            0x20..=0x7E => {r.push(String::from_utf8(vec![c]).unwrap())}
            x => {r.push(format!("/{x:X}"))}
        }
    }
    r.iter().map(|c| c.chars()).flatten().collect()
}


fn flatten_announce_list(announce_list: &[Bencode]) -> Vec<String>{
    announce_list
        .iter()
        .flat_map(|s| match s {
            Bencode::List(v) => v,
            _ => panic!("benis"),
        })
        .map(|message| match message {
            Bencode::Message(s) => String::from_utf8(s.to_vec()).expect("Cant convert url into string"),
            _ => panic!("Non message bencode in announce list"),
        }).collect()
}

fn _get_tracker_url(src: &BTreeMap<Vec<u8>, Bencode>) -> String {
    if let Some(url_bencode) = src.get("announce".as_bytes()) {
        if let Bencode::Message(url) = url_bencode {
            return String::from_utf8(url.clone()).expect("Invalid Url in announce field");
        }
        panic!("Value under \"announce\" key is not a bencoded string.")
    } else {
        panic!("Ill formatted torrent file, no \"announce\" key in root dictionary.");
    }
}
