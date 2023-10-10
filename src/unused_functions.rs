fn _flatten_announce_list(announce_list: &[Bencode]) -> Vec<String>{
    announce_list
        .iter()
        .flat_map(|s| match s {
            Bencode::List(v) => v,
            _ => panic!("benis"),
        })
        .map(|message| match message {
            Bencode::Message(s) => String::from_utf8(s.to_vec()).expect("Cant convert url into string"),
            _ => panic!("Non message bencode in announce list"),
        }).collect()
}

pub fn decode_int(byte_string: &mut Peekable<Iter<'_, u8>>) -> Result<Bencode, std::io::Error> {
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
        let err_msg = "Integer does not have an ending e.";
        return Err(std::io::Error::new(ErrorKind::InvalidData,err_msg)); 
    }
    let mut acc: isize = literals
        .iter()
        .fold(0usize, |acc, elem| flatten_number_step(acc, *elem)?)
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
    Bencode::Int(acc)
}