pub mod decode {
    use std::collections::HashMap;
    use crate::bencode::BencodeVal;
    type Bencode = crate::bencode::BencodeVal;
    type PeekIter<'a> = std::iter::Peekable<std::str::CharIndices<'a>>;
    pub fn decode_int<'a>(
        chars_indices: &mut PeekIter<'a>,
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

    pub fn decode_message<'a>(
        chars_indices: &mut PeekIter<'a>,
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

    pub fn decode_list<'a>(
        chars_indices: &mut PeekIter<'a>,
    ) -> BencodeVal {
        let mut vals: Vec<BencodeVal> = Vec::new();
        let mut seen_stop = false;
        while chars_indices.peek().is_some() {
            match Bencode::decode_single(chars_indices) {
                BencodeVal::Stop => {
                    seen_stop = true;
                    break;
                }
                x => vals.push(x),
            }
        }
        if !seen_stop {
            panic!("List does not have terminal value")
        };
        BencodeVal::List(vals)
    }

    pub fn decode_dict<'a>(
        chars_indices: &mut PeekIter<'a>,
    ) -> BencodeVal {
        // Keep keys seperate for sorted check at end
        let mut keys = Vec::<String>::new();
        let mut vals = Vec::<BencodeVal>::new();
        while chars_indices.peek().is_some() {
            let key = match Bencode::decode_single(chars_indices) {
                BencodeVal::Message(s) => s,
                BencodeVal::Stop => break,
                other => {
                    panic!(
                        "Wrong type of BencodeVal returned, \n Returned: {:?}",
                        other
                    )
                }
            };
            let val = match Bencode::decode_single(chars_indices) {
                BencodeVal::Stop => {
                    panic!("Key has no matching value. \n Lone key: {}", key.clone())
                }
                other => other,
            };
            keys.push(key);
            vals.push(val);
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
        BencodeVal::Dict(HashMap::<String, BencodeVal>::from_iter(std::iter::zip(
            keys, vals,
        )))
    }
}
