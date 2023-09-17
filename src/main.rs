use bit_tor::bencode::Bencode;
use std::fs;
use std::io::prelude::*;

// use std::net::TcpListener;
// use std::net::TcpStream;
// use std::thread;
// use std::time::Duration;

fn main() {
    let mut file = fs::File::open("/home/avi/rust_prac/bit_tor/src/big-buck-bunny.torrent").unwrap();
    let mut buf = Vec::<u8>::new();
    file.read(&mut buf).unwrap();
    println!("{:#?}",Bencode::decode_all(&buf)); 


}
