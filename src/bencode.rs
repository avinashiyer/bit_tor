use crate::decode::{decode_dict, decode_int, decode_list, decode_message};
use core::panic;
use std::collections::BTreeMap;
use std::fmt::{Write, Display};
use std::iter::Peekable;
use std::slice::Iter;
#[derive(Debug, PartialEq)]
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
        let mut count = 0;
        println!("Entered Decode All");
        while it.peek().is_some() {
            count += 1;
            println!("{count}");
            match Self::decode_single(&mut it) {
                Bencode::Stop => break,
                x => vals.push(x),
            }
        }
        vals
    }
    pub fn decode_single(
        byte_string: &mut Peekable<Iter<'_,u8>>,
    ) -> Bencode {
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

    pub fn encode_val(&self) -> String {
        match self {
            Bencode::Int(i) => format!("i{i}e"),
            Bencode::Message(s) => {
                let s_str = String::from_utf8_lossy(s);
                let s_str_len = s_str.len();
                format!("{}:{}", s_str_len, s_str)}
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

    fn encode_dict(d_ref: &BTreeMap<Vec<u8>, Bencode>) -> String {
        let mut res = String::new();
        write!(&mut res, "d").unwrap();
        for (k, v) in d_ref.iter() {
            let k_str = String::from_utf8_lossy(k);
            let key_len = k_str.len();
            write!(&mut res, "{key_len}:{k_str}{}", Self::encode_val(v)).unwrap();
        }
        write!(&mut res, "e").unwrap();
        res
    }
}

impl Display for Bencode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Bencode::Dict(d) => {
                write!(f,"Dict{{\n").expect("Error Displaying Bencode {self:?}");
                for (k,v) in d {
                    write!(f,"\t{} : {},\n",String::from_utf8_lossy(k),v).expect("Error Displaying Bencode {self:?}");
                }
                write!(f,"}}\n")
            }
            Bencode::Int(i) => {write!(f,"Int({})",i)}
            Bencode::List(l) => {
                write!(f,"List[\n").expect("Error Displaying Bencode {self:?}");
                for v in l {
                    write!(f,"\t{},\n",v).expect("Error Displaying Bencode {self:?}");
                }
                write!(f,"]\n")
        }
            Bencode::Message(s) => {write!(f,"Message({})",String::from_utf8_lossy(s))}
            Bencode::Stop => {write!(f,"Stop")}
        }
    }
}
