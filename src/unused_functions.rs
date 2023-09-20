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