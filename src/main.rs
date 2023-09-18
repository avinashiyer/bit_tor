 use bit_tor::bencode::Bencode;
use std::fs;
use std::io::prelude::*;

// use std::net::TcpListener;
// use std::net::TcpStream;
// use std::thread;
// use std::time::Duration;

fn main() {
    let file = fs::File::open("/home/avi/rust_prac/bit_tor/src/big-buck-bunny.torrent").unwrap();
    let f_iter: Vec<u8> = file.bytes().map(|e| e.expect("AHHHHHH")).collect();
    for v in Bencode::decode_all(&f_iter) {
        println!("{v}");
    } 

}
