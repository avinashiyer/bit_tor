mod bencode {
    use std::collections::HashMap;
    #[derive(Debug)]
    pub enum BencodeVal {
        Message(String),
        Int(isize),
        List(Vec<BencodeVal>),
        Dict(HashMap<String, BencodeVal>),
        Stop,
    }

    impl BencodeVal {
        // Convenience method to decode a whole string and return all bencode values in a vec
        pub fn decode_str(src: &str) -> Vec<BencodeVal> {
            let mut vals = Vec::<BencodeVal>::new();
            let mut chars_indices = src.char_indices().peekable();
            while let Some((_pos, ch_ref)) = chars_indices.peek() {
                match Self::decode_dispatch(&mut chars_indices) {
                    BencodeVal::Stop => break,
                    x => vals.push(x),
                }
            }
            vals
        }
        pub fn decode_dispatch<'a>(
            mut chars_indices: &mut std::iter::Peekable<std::str::CharIndices<'a>>,
        ) -> BencodeVal {
            let mut val = BencodeVal::Stop;
            while let Some((_pos, ch)) = chars_indices.peek() {
                val = match ch {
                    'e' => BencodeVal::Stop,
                    'i' => {
                        chars_indices.next();
                        Self::decode_int(&mut chars_indices)
                    }
                    '0'..='9' => Self::decode_message(&mut chars_indices),
                    'l' => {
                        chars_indices.next();
                        Self::decode_list(&mut chars_indices)
                    }
                    _ => {
                        panic!("Strange value in decode dispatch matched: {ch}")
                    }
                };
            }
            val
        }

        pub fn encode_str(src: BencodeVal) -> String {
            String::from("pass")
        }
        fn decode_int<'a>(
            chars_indices: &mut std::iter::Peekable<std::str::CharIndices<'a>>,
        ) -> BencodeVal {
            let mut last_matched: char = '\0';
            let s: String = chars_indices
                .by_ref()
                .take_while(|(_pos, c)| {
                    last_matched = *c;
                    *c != 'e'
                })
                .map(|(_pos, c)| c)
                .collect();
            if last_matched != 'e' {
                panic!()
            }
            BencodeVal::Int(s.parse::<isize>().unwrap())
        }

        fn decode_message<'a>(
            chars_indices: &mut std::iter::Peekable<std::str::CharIndices<'a>>,
        ) -> BencodeVal {
            let num = chars_indices
                .by_ref()
                .take_while(|(_pos, c)| *c != ':')
                .map(|(_pos, c)| c)
                .collect::<String>()
                .parse::<usize>()
                .unwrap();
            // Take num chars from iter
            println!("{num}");
            let s: String = chars_indices.take(num).map(|(_pos, c)| c).collect();
            if s.len() != num {
                panic!(
                    "String passed did not have number of chars specified.
                    \nExpected: {num} Actual: {}",
                    s.len()
                );
            }
            BencodeVal::Message(s)
        }

        fn decode_list<'a>(
            chars_indices: &mut std::iter::Peekable<std::str::CharIndices<'a>>,
        ) -> BencodeVal {
            let mut vals: Vec<BencodeVal> = Vec::new();
            while chars_indices.peek().is_some() {
                match Self::decode_dispatch(chars_indices) {
                    BencodeVal::Stop => break,
                    x => vals.push(x),
                }
            }
            BencodeVal::List(vals)
        }

        fn decode_dict<'a>(
            chars_indices: &mut std::iter::Peekable<std::str::CharIndices<'a>>,
        ) -> BencodeVal {
            let map = HashMap::<String, BencodeVal>::new();
            // Keep keys seperate for sorted check at end
            let mut keys = Vec::<String>::new();
            let mut vals = Vec::<BencodeVal>::new();
            while chars_indices.peek().is_some() {
                let key = match Self::decode_dispatch(chars_indices) {
                    BencodeVal::Message(s) => s,
                    other => {
                        panic!(
                            "Wrong type of BencodeVal returned, \n Returned: {:?}",
                            other
                        )
                    }
                };
                keys.push(key);
                vals.push(Self::decode_dispatch(chars_indices));
            }
            let raws: Vec<&[u8]> = keys.iter().map(|s: &String| s.as_bytes()).collect();
            if raws.len() > 1 {
                let asc = raws[0] < raws[1];
                if asc {
                    if !raws.windows(2).all(|w| w[0] <= w[1]) {
                        panic!("Unsorted keys.")
                    }
                } else {
                    if !raws.windows(2).all(|w| w[0] >= w[1]) {
                        panic!("Unsorted keys.")
                    }
                }
            }
            BencodeVal::Dict(map)
        }
    }
}
#[cfg(test)]
mod bencode_tests {
    use super::bencode::BencodeVal;
    #[test]
    fn decode_pos_int() {
        let num = 98;
        let s = format!("i{num}e");
        let vals = BencodeVal::decode_str(&s);
        if vals.len() != 1 {
            panic!(
                "Returned Wrong amount of values. \nExpected: 1 Actual: {}.",
                vals.len()
            )
        }
        match vals[0] {
            BencodeVal::Int(i) => {
                assert_eq!(i, num)
            }
            _ => panic!(
                "Wrong variant/type returned from decode_str. Value={:?}",
                vals[0]
            ),
        }
    }

    #[test]
    #[should_panic]
    fn bad_terminal_int_decode() {
        let s = String::from("i-45");
        BencodeVal::decode_dispatch(&mut s.char_indices().peekable());
    }

    #[test]
    fn test_message_decode() {
        let s = String::from("12:Hello World!");
        let res = BencodeVal::decode_dispatch(&mut s.char_indices().peekable());
        match res {
            BencodeVal::Message(x) => {
                assert_eq!(s[3..], x)
            }
            _ => {
                assert!(false)
            }
        }
    }

    // #[test]
    // fn test_sorted_checker_p() {
    //     let v = vec!["abc","acb","add","b"].iter().map(|s| s.as_bytes());
    //     assert!(BencodeVal::)
    // }
}
