use crate::decode::{decode_dict, decode_int, decode_list, decode_message};
use core::panic;
use std::collections::BTreeMap;
use std::fmt::Write;
#[derive(Debug, PartialEq)]
pub enum Bencode {
    Message(String),
    Int(isize),
    List(Vec<Bencode>),
    Dict(BTreeMap<String, Bencode>),
    Stop,
}

impl Bencode {
    // Convenience method to decode a whole string and return all bencode values in a vec
    pub fn decode_all(src: &str) -> Vec<Bencode> {
        let mut vals = Vec::<Bencode>::new();
        let mut chars_indices = src.char_indices().peekable();
        while chars_indices.peek().is_some() {
            match Self::decode_single(&mut chars_indices) {
                Bencode::Stop => break,
                x => vals.push(x),
            }
        }
        vals
    }
    pub fn decode_single(
        chars_indices: &mut std::iter::Peekable<std::str::CharIndices<'_>>,
    ) -> Bencode {
        let mut val = Bencode::Stop;
        if let Some((_pos, ch)) = chars_indices.peek() {
            val = match ch {
                'e' => Bencode::Stop,
                'i' => {
                    chars_indices.next();
                    decode_int(chars_indices)
                }
                '0'..='9' => decode_message(chars_indices),
                'l' => {
                    chars_indices.next();
                    decode_list(chars_indices, Vec::<Bencode>::new())
                }
                'd' => {
                    chars_indices.next();
                    decode_dict(chars_indices, BTreeMap::<String, Bencode>::new())
                }
                _ => {
                    panic!("Strange value in decode dispatch matched: {ch}")
                }
            };
        }
        val
    }

    pub fn encode_val(&self) -> String {
        match self {
            Bencode::Int(i) => format!("i{i}e"),
            Bencode::Message(s) => format!("{}:{s}", s.len()),
            Bencode::List(l) => Self::encode_list(l),
            Bencode::Dict(d) => Self::encode_dict(d),
            Bencode::Stop => panic!("Stop val passed to encode_val."),
        }
    }
    fn encode_list(v_ref: &Vec<Bencode>) -> String {
        let mut res = String::new();
        write!(&mut res, "l").unwrap();
        for val in v_ref {
            write!(&mut res, "{}", (val).encode_val()).unwrap();
        }
        write!(&mut res, "e").unwrap();
        res
    }

    fn encode_dict(d_ref: &BTreeMap<String, Bencode>) -> String {
        let mut res = String::new();
        write!(&mut res, "d").unwrap();
        for (k, v) in d_ref.iter() {
            let key_len = k.len();
            write!(&mut res, "{key_len}:{k}{}", Self::encode_val(v)).unwrap();
        }
        write!(&mut res, "e").unwrap();
        res
    }
}
