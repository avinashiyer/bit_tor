use crate::bencode::Bencode;
use core::panic;
use std::cmp::Ordering;
use std::collections::BTreeMap;
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
    let mut neg = false;
    let mut first_char: Option<u8> = None;
    if let Some(val) = byte_string.peek() {
        first_char = Some(**val);
        if **val == b'-' {
            byte_string.next();
            neg = true;
        }
    }
    let literals: Vec<u8> = byte_string
        .take_while(|c| {
            last_matched = **c;
            last_matched != b'e'
        })
        .map(|c| *c - b'0')
        .collect();
    if last_matched != b'e' {
        panic!()
    }
    let mut acc: isize = literals
        .iter()
        .fold(0usize, |acc, elem| flatten_number_step(acc, *elem))
        .try_into()
        .unwrap();
    if neg {
        acc = -acc;
    }
    if let Some(c) = first_char {
        if c == b'0' && literals.len() > 1 {
            panic!("leading zeroes found in decode_int.")
        }
        if neg && acc == 0 {
            panic!("Negative 0 found in decode_int")
        }
    }
    println!("INTEGER NUM: {acc}");
    Bencode::Int(acc)
}

fn flatten_number_step(acc: usize, elem: u8) -> usize {
    match elem {
        0..=9 => acc * 10 + (elem as usize),
        x => panic!(
            "Non ascii digit character within integer string. Namely {}",
            (x + b'0') as char
        ),
    }
}

pub fn decode_message(byte_string: &mut Peekable<Iter<'_, u8>>) -> Bencode {
    let num = byte_string
        .take_while(|c| **c != b':')
        .map(|c| *c - b'0')
        .fold(0usize, flatten_number_step);
    println!("MESSAGE NUM: {num}");
    // Take num chars from iter
    let s: Vec<u8> = byte_string.take(num).copied().collect();
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
    let mut key_val_pairs = Vec::<(Vec<u8>, Bencode)>::new();
    let mut seen = Vec::<u8>::new();
    while let Some(ch) = byte_string.peek() {
        seen.push(**ch);
        match ch {
            b'e' => {
                byte_string.next();
                check_keys_sorted(&key_val_pairs);
                for (k,v) in key_val_pairs {
                    if parent.insert(k, v).is_some() {
                        panic!("Duplicate key");
                    }
                }
                return Bencode::Dict(parent)
            }
            c if !c.is_ascii_digit() => {
                let rest_of_iter:Vec<u8> = byte_string.map(|c|*c).collect();

                panic!(
                    "Ill formatted key within dictionary. \nIter Dump: {}",
                    helper(rest_of_iter))
            }
            _ => {}
        }
        let key;
        if let Bencode::Message(k) = decode_message(byte_string) {
            key = k;
        } else {
            panic!("decode message returned a strange bencode. Expected to return Bencode::Message")
        }
        let val = get_value(byte_string, &key);
        key_val_pairs.push((key, val));
    }
    Bencode::Stop
}


pub fn helper(v:Vec<u8>) -> String{
    let mut r:Vec<String> = Vec::new();
    for c in v {
        match c {
            0x20..=0x7E => {r.push(String::from_utf8(vec![c]).unwrap())}
            x => {r.push(format!("/{x:X}"))}
        }
    }
    r.iter().map(|c| c.chars()).flatten().collect()
}


fn check_keys_sorted(key_val_pairs: &Vec<(Vec<u8>, Bencode)>) {
    if key_val_pairs.len() > 2 {
        let key_ordering = key_val_pairs[0].0.cmp(&key_val_pairs[1].0);
        let is_sorted = (0..key_val_pairs.len() - 1)
            .all(|i| check_sorted(key_val_pairs, i, key_ordering));
        if !is_sorted {panic!("Unsorted keys in dictionary")}
    }
}

// https://rust-lang.github.io/rfcs/2351-is-sorted.html
fn check_sorted(key_val_pairs: &[(Vec<u8>, Bencode)], i: usize, key_ordering: Ordering) -> bool {
    key_val_pairs[i].0.cmp(&key_val_pairs[i + 1].0) == key_ordering
}

fn get_value(byte_string: &mut Peekable<Iter<'_, u8>>, key: &[u8]) -> Bencode {
    let val = match byte_string.peek() {
        Some(ch) => match ch {
            b'd' => {
                byte_string.next();
                decode_dict(byte_string, BTreeMap::<Vec<u8>, Bencode>::new())
            }
            b'e' => {
                panic!(
                    "{} has no corresponding value.",
                    String::from_utf8_lossy(key)
                )
            }
            _ => Bencode::decode_single(byte_string),
        },
        None => {
            panic!(
                "{} has no corresponding value.",
                String::from_utf8_lossy(key)
            )
        }
    };
    val
}
