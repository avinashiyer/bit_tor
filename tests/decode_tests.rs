mod decode_tests {
    use bit_tor::bencode::Bencode;
    use std::collections::BTreeMap;
    #[test]
    fn decode_pos_int() {
        let num = 98;
        let s = format!("i{num}e");
        let vals = Bencode::decode_all(&s);
        if vals.len() != 1 {
            panic!(
                "Returned Wrong amount of values. \nExpected: 1 Actual: {}.",
                vals.len()
            )
        }
        match vals[0] {
            Bencode::Int(i) => {
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
        Bencode::decode_single(&mut s.char_indices().peekable());
    }

    #[test]
    fn neg_int_decode() {
        let num = -32;
        let s = format!("i{num}e");
        match Bencode::decode_single(&mut s.char_indices().peekable()) {
            Bencode::Int(n) => assert_eq!(n, num),
            _ => {
                assert!(false)
            }
        }
    }

    #[test]
    fn test_message_decode() {
        let s = String::from("12:Hello World!");
        let res = Bencode::decode_single(&mut s.char_indices().peekable());
        match res {
            Bencode::Message(x) => {
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
        Bencode::decode_single(&mut s.char_indices().peekable());
    }

    #[test]
    fn test_message_with_bencoded_vals() {
        let s = "35:abcd12:hello_world!li22ed3:eari45ee";
        match Bencode::decode_single(&mut s.char_indices().peekable()) {
            Bencode::Message(val) => assert_eq!(s[3..], val),
            _ => {
                assert!(false)
            }
        }
    }

    #[test]
    fn test_decode_multiple_vals_ints() {
        let s = "i33ei-1e";
        let v = Bencode::decode_all(s);
        println!("{:?}", v);
        match v[0] {
            Bencode::Int(x) => assert_eq!(33, x),
            _ => {
                assert!(false)
            }
        }
        match v[1] {
            Bencode::Int(x) => assert_eq!(-1, x),
            _ => assert!(false),
        }
    }

    #[test]
    fn test_decode_list_pos() {
        let mut s = "li3e5:hi55ei8e0:e".char_indices().peekable();
        let v = Bencode::decode_single(&mut s);
        let vec = match &v {
            Bencode::List(val) => val,
            _ => panic!("{:?}", v),
        };
        let exp = Bencode::List(vec![
            Bencode::Int(3),
            Bencode::Message("hi55e".to_owned()),
            Bencode::Int(8),
            Bencode::Message("".to_owned()),
        ]);
        // Checks if equality checks between lists is working
        assert_eq!(vec[0], Bencode::Int(3));
        assert_eq!(vec[1], Bencode::Message(String::from("hi55e")));
        assert_eq!(vec[2], Bencode::Int(8));
        assert_eq!(vec[3], Bencode::Message(String::from("")));
        assert_eq!(v, exp);
    }

    #[test]
    #[should_panic]
    fn test_decode_list_no_end() {
        let mut s = "li44ei4e4:abcd".char_indices().peekable();
        Bencode::decode_single(&mut s);
    }

    #[test]
    fn test_nested_list_decode() {
        let mut s = "li132e2:2:0:li44e0:l4:spamei22ei23ee1:fe"
            .char_indices()
            .peekable();
        let real = Bencode::decode_single(&mut s);
        let exp = Bencode::List(vec![
            Bencode::Int(132),
            Bencode::Message("2:".to_owned()),
            Bencode::Message("".to_owned()),
            Bencode::List(vec![
                Bencode::Int(44),
                Bencode::Message("".to_owned()),
                Bencode::List(vec![Bencode::Message("spam".to_owned())]),
                Bencode::Int(22),
                Bencode::Int(23),
            ]),
            Bencode::Message("f".to_owned()),
        ]);
        println!("{}", exp.encode_val());
        assert_eq!(real, exp)
    }

    #[test]
    fn test_nested_list_str() {
        let s = "llleee";
        assert_eq!(
            Bencode::decode_all(s)[0],
            Bencode::List(vec![Bencode::List(vec![Bencode::List(
                Vec::<Bencode>::new()
            )])])
        )
    }

    #[test]
    fn test_dict_decode_pos() {
        let mut s = "d3:cow3:moo4:spam4:eggse".char_indices().peekable();
        let vals = BTreeMap::from([
            (String::from("cow"), Bencode::Message(String::from("moo"))),
            (String::from("spam"), Bencode::Message(String::from("eggs"))),
        ]);
        let x = Bencode::decode_single(&mut s);
        if let Bencode::Dict(map) = x {
            assert_eq!(map, vals);
        } else {
            assert!(false, "{:?}", x);
        }
    }

    #[test]
    fn test_nested_dict_decode() {
        let mut s = "d0:de1:adee".char_indices().peekable();
        let exp = Bencode::Dict(BTreeMap::from([
            (
                String::from(""),
                Bencode::Dict(BTreeMap::<String, Bencode>::new()),
            ),
            (
                String::from("a"),
                Bencode::Dict(BTreeMap::<String, Bencode>::new()),
            ),
        ]));
        assert_eq!(Bencode::decode_single(&mut s), exp);
    }

    #[test]
    fn test_3_nested_dict_decode() {
        let mut s = "d2:aad3:bfgdeee".char_indices().peekable();
        let exp = Bencode::Dict(BTreeMap::from([(
            "aa".to_owned(),
            Bencode::Dict(BTreeMap::from([(
                "bfg".to_owned(),
                Bencode::Dict(BTreeMap::<String, Bencode>::new()),
            )])),
        )]));
        assert_eq!(Bencode::decode_single(&mut s),exp)
    }
}
