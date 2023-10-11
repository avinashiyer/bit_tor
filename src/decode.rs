use once_cell::sync::Lazy;
use regex::bytes::Regex;

use crate::bencode::Bencode;
use std::cmp::Ordering;
use std::collections::BTreeMap;
use std::iter::Peekable;
use std::slice::Iter;

use crate::{escape_u8_slice, make_bad_data_err};

///Assumes iterator starts at the number (i.e. caller consumed the 'i')
///Errors
///     1) No terminal 'e' within iterator
///     2) String can not be parsed as an int (excluding e)
///     3) Leading Zero/es
///     4) Negative Zero
pub fn decode_int(byte_string: &mut Peekable<Iter<'_, u8>>) -> Result<Bencode, std::io::Error> {
    let mut relevant_bytes: Vec<u8> = Vec::new();
    while byte_string.peek().is_some() {
        let ch = byte_string.next().unwrap();
        relevant_bytes.push(*ch);
        if *ch == b'e' {
            break;
        }
    }
    validate_and_parse_int(relevant_bytes)
}

fn validate_and_parse_int(bytes: Vec<u8>) -> Result<Bencode, std::io::Error> {
    // once_cell::Lazy allows this regex expression to only need compilation once (on first use)
    static RE: Lazy<Regex> = Lazy::new(|| Regex::new(r"^(-?[1-9][0-9]*)e$|^(0)e$").unwrap());
    let Some(captures) = RE.captures(&bytes) else {
        let err_msg = format!(
            "Malformed integer value passed to decoder: {}",
            escape_u8_slice(&bytes)
        );
        return Err(make_bad_data_err(&err_msg));
    };
    let captured_str = std::str::from_utf8(&captures[1]).unwrap();
    let num = match str::parse::<isize>(captured_str) {
        Ok(num) => num,
        Err(_) => {
            let err_msg = format!("Error parsing int passed to decoder: {captured_str}");
            return Err(make_bad_data_err(&err_msg));
        }
    };
    Ok(Bencode::Int(num))
}

pub fn decode_message(byte_string: &mut Peekable<Iter<'_, u8>>) -> Result<Bencode, std::io::Error> {
    let number_bytes = byte_string.take_while(|c| **c != b':').copied().collect();
    let number_str = match String::from_utf8(number_bytes) {
        Ok(s) => s,
        Err(_) => {
            return Err(make_bad_data_err(
                "Unexpected bytes in message length string",
            ))
        }
    };
    let num = match str::parse::<isize>(&number_str) {
        Ok(n) => n,
        Err(_) => {
            return Err(make_bad_data_err(&format!(
                "Error parsing isize from str: {}",
                &number_str
            )))
        }
    };
    if num.is_negative() {
        let err_msg = format!("Negative Message Length String: {num}");
        return Err(make_bad_data_err(&err_msg));
    }
    let num = num as usize;
    // Take {num} chars from iter
    let s: Vec<u8> = byte_string.take(num).copied().collect();
    if s.len() != num {
        let err_msg = format!(
            "String passed did not have number of bytes specified.
            \nExpected: {num} Actual: {}\n
            {:?}",
            s.len(),
            s
        );
        return Err(make_bad_data_err(&err_msg));
    }
    Ok(Bencode::Message(s))
}

pub fn decode_list(
    byte_string: &mut Peekable<Iter<'_, u8>>,
    mut parent: Vec<Bencode>,
) -> Result<Bencode, std::io::Error> {
    while let Some(ch) = byte_string.peek() {
        match ch {
            b'l' => {
                byte_string.next();
                parent.push(decode_list(byte_string, Vec::<Bencode>::new())?)
            }
            b'e' => {
                byte_string.next();
                return Ok(Bencode::List(parent));
            }
            _ => parent.push(Bencode::decode_dispatch(byte_string)?),
        }
    }
    Err(make_bad_data_err("No terminal b'e' found in encoded value"))
}

pub fn decode_dict(
    byte_string: &mut Peekable<Iter<'_, u8>>,
    mut parent: BTreeMap<Vec<u8>, Bencode>,
) -> Result<Bencode, std::io::Error> {
    let mut key_val_pairs = Vec::<(Vec<u8>, Bencode)>::new();
    while let Some(ch) = byte_string.peek() {
        match ch {
            b'e' => {
                byte_string.next();
                if !check_keys_sorted(&key_val_pairs) {
                    return Err(make_bad_data_err("Keys in dictionary are unsorted"));
                }
                for (k, v) in key_val_pairs {
                    if let std::collections::btree_map::Entry::Vacant(e) = parent.entry(k.clone()) {
                        e.insert(v);
                    } else {
                        let err_msg = format!(
                            "Duplicate keys in encoded dictionary. Offender: {}",
                            escape_u8_slice(&k)
                        );
                        return Err(make_bad_data_err(&err_msg));
                    } 
                }
                return Ok(Bencode::Dict(parent));
            }
            c if !c.is_ascii_digit() => {
                let rest_of_iter: Vec<u8> = byte_string.copied().collect();
                let err_msg = format!(
                    "Ill formatted key within dictionary. \nIter Dump: {}",
                    escape_u8_slice(&rest_of_iter)
                );
                return Err(make_bad_data_err(&err_msg));
            }
            _ => { /*leave match*/ }
        }
        let key;
        if let Bencode::Message(k) = decode_message(byte_string)? {
            key = k;
        } else {
            return Err(make_bad_data_err("Dictionary key is not a string"));
        }
        let val = get_value(byte_string, &key)?;
        key_val_pairs.push((key, val));
    }
    Ok(Bencode::Stop)
}

fn check_keys_sorted(key_val_pairs: &Vec<(Vec<u8>, Bencode)>) -> bool {
    if key_val_pairs.len() < 3 {
        return true;
    }
    let key_ordering = key_val_pairs[0].0.cmp(&key_val_pairs[1].0);
    (0..key_val_pairs.len() - 1).all(|i| check_sorted(key_val_pairs, i, key_ordering))
}

// Source: https://rust-lang.github.io/rfcs/2351-is-sorted.html
fn check_sorted(key_val_pairs: &[(Vec<u8>, Bencode)], i: usize, key_ordering: Ordering) -> bool {
    key_val_pairs[i].0.cmp(&key_val_pairs[i + 1].0) == key_ordering
}

fn get_value(
    byte_string: &mut Peekable<Iter<'_, u8>>,
    key: &[u8],
) -> Result<Bencode, std::io::Error> {
    let val = match byte_string.peek() {
        Some(ch) => match ch {
            b'd' => {
                byte_string.next();
                decode_dict(byte_string, BTreeMap::<Vec<u8>, Bencode>::new())?
            }
            b'e' => {
                let err_msg = format!(
                    "{} has no corresponding value. (dictionary ends early)",
                    escape_u8_slice(key)
                );
                return Err(make_bad_data_err(&err_msg));
            }
            _ => Bencode::decode_dispatch(byte_string)?,
        },
        None => {
            let err_msg = format!(
                "Key: {} has no corresponding value (Iterator Empty)",
                escape_u8_slice(key)
            );
            return Err(make_bad_data_err(&err_msg));
        }
    };
    Ok(val)
}
