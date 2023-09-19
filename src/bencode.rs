use crate::decode::{decode_dict, decode_int, decode_list, decode_message};
use core::panic;
use std::collections::BTreeMap;
use std::fmt::{self};
use std::iter::Peekable;
use std::slice::Iter;
#[derive(PartialEq)]
pub enum Bencode {
    Message(Vec<u8>),
    Int(isize),
    List(Vec<Bencode>),
    Dict(BTreeMap<Vec<u8>, Bencode>),
    Stop,
}

impl Bencode {
    // Convenience method to decode a whole string and return all bencode values in a vec
    pub fn decode_all(src: &[u8]) -> Vec<Bencode> {
        let mut vals = Vec::<Bencode>::new();
        let mut it = src.iter().peekable();
        while it.peek().is_some() {
            match Self::decode_single(&mut it) {
                Bencode::Stop => break,
                x => vals.push(x),
            }
        }
        vals
    }
    pub fn decode_single(byte_string: &mut Peekable<Iter<'_, u8>>) -> Bencode {
        let mut val = Bencode::Stop;
        if let Some(ch) = byte_string.peek() {
            val = match **ch {
                b'e' => Bencode::Stop,
                b'i' => {
                    byte_string.next();
                    decode_int(byte_string)
                }
                b'0'..=b'9' => decode_message(byte_string),
                b'l' => {
                    byte_string.next();
                    decode_list(byte_string, Vec::<Bencode>::new())
                }
                b'd' => {
                    byte_string.next();
                    decode_dict(byte_string, BTreeMap::<Vec<u8>, Bencode>::new())
                }
                _ => {
                    panic!("Strange value in decode dispatch matched: {ch}")
                }
            };
        }
        val
    }

    pub fn encode_val(&self) -> Vec<u8> {
        match self {
            Bencode::Int(i) => format!("i{i}e").as_bytes().to_vec(),
            Bencode::Message(s) => encode_message(s),
            Bencode::List(l) => Self::encode_list(l),
            Bencode::Dict(d) => Self::encode_dict(d),
            Bencode::Stop => panic!("Stop val passed to encode_val."),
        }
    }

    fn encode_list(v_ref: &Vec<Bencode>) -> Vec<u8> {
        let mut res = Vec::new();
        res.push(b'l');
        for val in v_ref {
            res.append(&mut val.encode_val())
        }
        res.push(b'e');
        res
    }

    fn encode_dict(d_ref: &BTreeMap<Vec<u8>, Bencode>) -> Vec<u8> {
        let mut res = Vec::new();
        res.push(b'd');
        for (k, v) in d_ref.iter() {
            let mut len = k.len().to_string().as_bytes().to_vec();
            res.append(&mut len);
            res.push(b':');
            res.append(&mut k.clone());
            res.append(&mut v.encode_val())
        }
        res.push(b'e');
        res
    }
}

fn encode_message(s: &Vec<u8>) -> Vec<u8> {
    let mut s_vec = s.len().to_string().as_bytes().to_vec();
    s_vec.push(b':');
    s_vec.append(&mut s.clone());
    s_vec
}

impl fmt::Debug for Bencode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Bencode::Dict(d) => {
                write!(f,"Dict")?;
                f.debug_map().entries(d.iter().map
                (|(k, v)| (String::from_utf8_lossy(k),v))).finish()
            }
            Bencode::Int(i) => {
                write!(f, "Int({})", i)
            }
            Bencode::List(l) => {
                writeln!(f, "List")?;
                f.debug_list().entries(l.iter()).finish()
            }
            Bencode::Message(s) => {
                write!(f, "Message({})", String::from_utf8_lossy(s))
            }
            Bencode::Stop => {
                write!(f, "Stop")
            }
        }
    }
}
