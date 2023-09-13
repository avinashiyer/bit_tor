pub mod decode;
pub mod bencode {
    use crate::decode::decode::{decode_dict, decode_int, decode_list, decode_message};
    use core::panic;
    use std::borrow::BorrowMut;
    use std::collections::HashMap;
    use std::fmt::{Write, write};
    #[derive(Debug, PartialEq)]
    pub enum BencodeVal {
        Message(String),
        Int(isize),
        List(Vec<BencodeVal>),
        Dict(HashMap<String, BencodeVal>),
        Stop,
    }

    impl BencodeVal {
        // Convenience method to decode a whole string and return all bencode values in a vec
        pub fn decode_all(src: &str) -> Vec<BencodeVal> {
            let mut vals = Vec::<BencodeVal>::new();
            let mut chars_indices = src.char_indices().peekable();
            while chars_indices.peek().is_some() {
                match Self::decode_single(&mut chars_indices) {
                    BencodeVal::Stop => break,
                    x => vals.push(x),
                }
            }
            vals
        }
        pub fn decode_single<'a>(
            mut chars_indices: &mut std::iter::Peekable<std::str::CharIndices<'a>>,
        ) -> BencodeVal {
            let mut val = BencodeVal::Stop;
            if let Some((_pos, ch)) = chars_indices.peek() {
                val = match ch {
                    'e' => BencodeVal::Stop,
                    'i' => {
                        chars_indices.next();
                        decode_int(&mut chars_indices)
                    }
                    '0'..='9' => decode_message(&mut chars_indices),
                    'l' => {
                        chars_indices.next();
                        decode_list(&mut chars_indices)
                    }
                    'd' => {
                        chars_indices.next();
                        decode_dict(&mut chars_indices)
                    }
                    _ => {
                        panic!("Strange value in decode dispatch matched: {ch}")
                    }
                };
            }
            val
        }

        pub fn encode_val(val_ref: &BencodeVal) -> String {
            match val_ref {
                BencodeVal::Int(i) => format!("i{i}e"),
                BencodeVal::Message(s) => format!("{}:{s}", s.len()),
                BencodeVal::List(l) => Self::encode_list(l),
                BencodeVal::Dict(d) => Self::encode_dict(d),
                BencodeVal::Stop => panic!("Stop val passed to encode_val."),
            }
        }
        fn encode_list(v_ref: &Vec<BencodeVal>) -> String {
            let mut res = String::new();
            write!(&mut res,"l").unwrap();
            for val in v_ref {
                write!(&mut res, "{}",Self::encode_val(&val)).unwrap();
            }
            write!(&mut res, "e").unwrap();
            res
        }

        fn encode_dict(d_ref: &HashMap<String,BencodeVal>) -> String {
            let mut res = String::new();
            write!(&mut res, "d").unwrap();
            for (k,v) in d_ref.iter() {
                let key_len = k.len();
                write!(&mut res, "{key_len}:{k}{}",Self::encode_val(v)).unwrap();
            }
            write!(&mut res, "e").unwrap();
            res
        }
    }
}
