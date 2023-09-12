#[allow(dead_code)]
mod bencode {
    use std::collections::HashMap;
    #[derive(Debug, PartialEq)]
    pub enum BencodeVal<'a> {
        Message(&'a str),
        Int(isize),
        List(Vec<BencodeVal<'a>>),
        Dict(HashMap<String, BencodeVal<'a>>),
        Stop,
    }

    impl BencodeVal<'a> {
        // Convenience method to decode a whole string and return all bencode values in a vec
        pub fn decode_str(src: &str) -> Vec<BencodeVal> {
            let mut vals = Vec::<BencodeVal>::new();
            let mut chars_indices = src.char_indices().peekable();
            while chars_indices.peek().is_some() {
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
            if let Some((_pos, ch)) = chars_indices.peek() {
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
                    'd' => {
                        chars_indices.next();
                        Self::decode_dict(&mut chars_indices)
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
    fn neg_int_decode() {
        let num = -32;
        let s = format!("i{num}e");
        match BencodeVal::decode_dispatch(&mut s.char_indices().peekable()) {
            BencodeVal::Int(n) => assert_eq!(n, num),
            _ => {
                assert!(false)
            }
        }
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

    #[test]
    #[should_panic]
    fn test_message_short() {
        let s = String::from("12:hello ");
        BencodeVal::decode_dispatch(&mut s.char_indices().peekable());
    }

    #[test]
    fn test_message_with_bencoded_vals() {
        let s = "35:abcd12:hello_world!li22ed3:eari45ee";
        match BencodeVal::decode_dispatch(&mut s.char_indices().peekable()) {
            BencodeVal::Message(val) => assert_eq!(s[3..], val),
            _ => {
                assert!(false)
            }
        }
    }

    #[test]
    fn test_decode_multiple_vals_ints() {
        let s = "i33ei-1e";
        let v = BencodeVal::decode_str(s);
        println!("{:?}", v);
        match v[0] {
            BencodeVal::Int(x) => assert_eq!(33, x),
            _ => {
                assert!(false)
            }
        }
        match v[1] {
            BencodeVal::Int(x) => assert_eq!(-1, x),
            _ => assert!(false),
        }
    }

    #[test]
    fn test_decode_list_pos() {
        let mut s = "li3e5:hi55ei8e0:e".char_indices().peekable();
        let v = BencodeVal::decode_dispatch(&mut s);
        let vec = match v {
            BencodeVal::List(val) => val,
            _ => panic!("{:?}", v),
        };
        assert_eq!(vec[0], BencodeVal::Int(3));
        assert_eq!(vec[1], BencodeVal::Message(String::from("hi55e")));
        assert_eq!(vec[2], BencodeVal::Int(8));
        assert_eq!(vec[3], BencodeVal::Message(String::from("")));
    }

    #[test]
    #[should_panic]
    fn test_decode_list_no_end() {
        let mut s = "li44ei4e4:abcd".char_indices().peekable();
        BencodeVal::decode_dispatch(&mut s);
    }
    // TODO: Nested List

    #[test]
    fn test_dict_decode_pos() {
        let mut s = "d3:cow3:moo4:spam4:eggse".char_indices().peekable();
        let vals = HashMap::from([
            ("cow",BencodeVal::Message("moo")),
        ])
    }

    // #[test]
    // fn test_sorted_checker_p() {
    //     let v = vec!["abc","acb","add","b"].iter().map(|s| s.as_bytes());
    //     assert!(BencodeVal::)
    // }
}
