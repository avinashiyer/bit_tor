use crate::bencode::Bencode;
use std::collections::BTreeMap;
type PeekIter<'a> = std::iter::Peekable<std::str::CharIndices<'a>>;

///Param: mutable ref to CharIndices iterator (consumes until 'e' terminal is found)
///       Assumes iterator starts at the number (i.e. caller consumed the 'i')
///Output: Bencode::Int({num}) where num is the parsed number
///Panics:
///     1) No terminal 'e' within stream
///     2) string is not parsable as an int (excluding e) 
pub fn decode_int(
    chars_indices: &mut PeekIter<'_>,
) -> Bencode{
    //TODO: add leading zeroes/ negative 0 check
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
    Bencode::Int(s.parse::<isize>().unwrap())
}

pub fn decode_message(chars_indices: &mut PeekIter<'_>) -> Bencode {
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
    Bencode::Message(s)
}
pub fn decode_list(chars_indices: &mut PeekIter<'_>, mut parent:Vec<Bencode>) -> Bencode {
    while let Some((_pos,ch)) = chars_indices.peek() {
        match ch {
            'l' => {chars_indices.next();parent.push(decode_list(chars_indices, Vec::<Bencode>::new()))}
            'e' => {chars_indices.next();return Bencode::List(parent);}
            _ => {parent.push(Bencode::decode_single(chars_indices))}
        }
    }
    panic!("No terminal???")
}

pub fn decode_dict(chars_indices: &mut PeekIter<'_>, parent:BTreeMap<String,Bencode>) -> Bencode {
    while let Some((_pos,ch)) = chars_indices.peek() {
        if !ch.is_ascii_digit() {
            panic!("Ill formatted key value in pair within dictionary. \nIter Dump: {:?}",chars_indices.collect());
        }
        let key;
        if let Bencode::Message(k) = decode_message(chars_indices) {
            key = k;
        } else {
            panic!("decode message returned a strange bencode. Expected to return Bencode::Message")
        }
        let match chars_indices.peek() {
            Some((_pos,ch))
        }

    }
    Bencode::Stop;
}
