pub mod decode;
pub mod bencode {
    use core::panic;
    use std::collections::HashMap;
    use crate::decode::decode::{decode_int,decode_dict,decode_message,decode_list};
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

        pub fn encode_str(_src: BencodeVal) -> String {
            String::from("pass")
        }
    }
}
