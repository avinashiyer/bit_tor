use crate::bencode::BencodeVal;
use std::collections::BTreeMap;
type Bencode = crate::bencode::BencodeVal;
type PeekIter<'a> = std::iter::Peekable<std::str::CharIndices<'a>>;

///Param: mutable ref to CharIndices iterator (consumes until 'e' terminal is found)
///       Assumes iterator starts at the number (i.e. caller consumed the 'i')
///Output: BencodeVal::Int({num}) where num is the parsed number
///Panics:
///     1) No terminal 'e' within stream
///     2) string is not parsable as an int (excluding e) 
pub fn decode_int(
    //TODO: add leading zeroes/ negative 0 check
    chars_indices: &mut PeekIter<'_>,
) -> BencodeVal{
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

pub fn decode_message(chars_indices: &mut PeekIter<'_>) -> BencodeVal {
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
// llelee
pub fn decode_list(chars_indices: &mut PeekIter<'_>, mut parent:Vec<BencodeVal>) -> BencodeVal {
    while let Some((_pos,ch)) = chars_indices.peek() {
        match ch {
            'l' => {chars_indices.next();parent.push(decode_list(chars_indices, Vec::<BencodeVal>::new()))}
            'e' => {chars_indices.next();return BencodeVal::List((parent));}
            _ => {parent.push(BencodeVal::decode_single(chars_indices))}
        }
    }
    panic!("No terminal???")
}

pub fn decode_dict(chars_indices: &mut PeekIter<'_>) -> BencodeVal {
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
        } else if !raws.windows(2).all(|w| w[0] >= w[1]) {
            panic!("Unsorted keys.")
        }
    }
    BencodeVal::Dict(BTreeMap::<String, BencodeVal>::from_iter(std::iter::zip(
        keys, vals,
    )))
}
