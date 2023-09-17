use crate::bencode::Bencode;
use core::panic;
use std::collections::BTreeMap;
use std::io::Read;
use std::iter::Peekable;
use std::slice::Iter;

///Param: mutable ref to CharIndices iterator (consumes until 'e' terminal is found)
///       Assumes iterator starts at the number (i.e. caller consumed the 'i')
///Output: Bencode::Int({num}) where num is the parsed number
///Panics:
///     1) No terminal 'e' within stream
///     2) string is not parsable as an int (excluding e)
pub fn decode_int(byte_string: &mut Peekable<Iter<'_, u8>>) -> Bencode {
    let mut last_matched: u8 = b'\0';
    let mut literals: Vec<u8> = byte_string
        .take_while(|c| {
            last_matched = **c;
            last_matched != b'e'
        })
        .map(|c| *c - b'0')
        .collect();
    if last_matched != b'e' {
        panic!()
    }
    let mut lit_iter = literals.iter();
    let first_char = literals.first();
    let mut neg:bool = false;
    if let Some(c) = first_char {if *c == b'-' {lit_iter.next();neg = true;}}
    let mut acc: isize = lit_iter.fold(0usize, |acc, elem| flatten_number_step(acc, *elem)).try_into().unwrap();
    if neg {acc = -acc;}
    if let Some(c) = first_char {
        if *c == b'0' && acc != 0 {panic!("leading zeroes found in decode_int.")}
        if neg && acc == 0 {panic!("Negative 0 found in decode_int")}
    }
    Bencode::Int(acc)
}

fn flatten_number_step(acc:usize, elem:u8) -> usize {
    match elem {
        0..=9 => return acc*10 + (elem as usize),
        x => panic!("Non ascii digit character within integer string. Namely {}",(x + b'0') as char),
    }
}

pub fn decode_message(byte_string: &mut Peekable<Iter<'_, u8>>) -> Bencode {
    let num = byte_string
        .take_while(|c| **c != b':')
        .map(|c| *c - b'0')
        .fold(0usize, |acc, elem| flatten_number_step(acc, elem));

    // Take num chars from iter
    let s: Vec<u8> = byte_string.take(num).map(|c| *c).collect();
    if s.len() != num {
        panic!(
            "String passed did not have number of bytes specified.
            \nExpected: {num} Actual: {}\n
            {:?}",
            s.len(),
            s
        );
    }
    Bencode::Message(s)
}
pub fn decode_list(byte_string: &mut Peekable<Iter<'_, u8>>, mut parent: Vec<Bencode>) -> Bencode {
    while let Some(ch) = byte_string.peek() {
        match ch {
            b'l' => {
                byte_string.next();
                parent.push(decode_list(byte_string, Vec::<Bencode>::new()))
            }
            b'e' => {
                byte_string.next();
                return Bencode::List(parent);
            }
            _ => parent.push(Bencode::decode_single(byte_string)),
        }
    }
    panic!("No terminal???")
}
pub fn decode_dict(
    byte_string: &mut Peekable<Iter<'_, u8>>,
    mut parent: BTreeMap<Vec<u8>, Bencode>,
) -> Bencode {
    while let Some(ch) = byte_string.peek() {
        match ch {
            b'e' => {
                byte_string.next();
                return Bencode::Dict(parent);
            }
            c if !c.is_ascii_digit() => {
                panic!(
                    "Ill formatted key value in pair within dictionary. \nIter Dump: {:?}",
                    byte_string.collect::<Vec<_>>()
                )
            }
            _ => {}
        }
        let key;
        if let Bencode::Message(k) = decode_message(byte_string) {
            key = k;
        } else {
            panic!("decode message returned a strange bencode. Expected to return Bencode::Message")
        }
        let val = match byte_string.peek() {
            Some(ch) => match ch {
                b'd' => {
                    byte_string.next();
                    decode_dict(byte_string, BTreeMap::<Vec<u8>, Bencode>::new())
                }
                b'e' => {
                    panic!("{} has no corresponding value.",String::from_utf8_lossy(&key))
                }
                _ => Bencode::decode_single(byte_string),
            },
            None => {
                panic!("{} has no corresponding value.",String::from_utf8_lossy(&key))
            }
        };
        if parent.insert(key, val).is_some() {
            panic!("Duplicate key passed to dict");
        }
    }
    Bencode::Stop
}
