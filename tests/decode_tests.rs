mod decode_tests {
    use bit_tor::bencode::Bencode;
    use std::{collections::BTreeMap, error::Error};

    #[test]
    fn decode_pos_int() {
        let num = 98;
        let s = format!("i{num}e").as_bytes().to_vec();
        let vals = Bencode::decode_all(&s).unwrap();
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
    fn bad_terminal_int_decode() {
        let s = String::from("i-45").as_bytes().to_vec();
        assert!(Bencode::decode_dispatch(&mut s.iter().peekable()).is_err());
    }

    #[test]
    fn neg_int_decode() {
        let num = -32;
        let s = format!("i{num}e").as_bytes().to_vec();
        match Bencode::decode_dispatch(&mut s.iter().peekable()).unwrap() {
            Bencode::Int(n) => assert_eq!(n, num),
            _ => {
                assert!(false)
            }
        }
    }

    #[test]
    fn leading_zeros() {
        let mut s = "i03e".as_bytes().iter().peekable();
        assert!(Bencode::decode_dispatch(&mut s).is_err());
    }

    #[test]
    fn neg_zero() {
        let mut s = "i-0e".as_bytes().iter().peekable();
        assert!(Bencode::decode_dispatch(&mut s).is_err());
    }

    #[test]
    fn leading_zeroes_zeroes() {
        let mut s = "i000e".as_bytes().iter().peekable();
        assert!(Bencode::decode_dispatch(&mut s).is_err());
    }

    #[test]
    fn leading_zeroes_zero() {
        let mut s = "i00e".as_bytes().iter().peekable();
        assert!(Bencode::decode_dispatch(&mut s).is_err());
    }

    #[test]
    fn leading_zeroes_neg_zero() {
        let mut s = "i-00e".as_bytes().iter().peekable();
        assert!(Bencode::decode_dispatch(&mut s).is_err());
    }

    #[test]
    fn test_message_decode() {
        let s = String::from("12:Hello World!").as_bytes().to_vec();
        let res = Bencode::decode_dispatch(&mut s.iter().peekable()).unwrap();
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
    fn test_message_short() {
        let s = "12:hello ".as_bytes().iter();
        assert!(Bencode::decode_dispatch(&mut s.peekable()).is_err());
    }

    #[test]
    fn test_message_with_bencoded_vals() {
        let s = "35:abcd12:hello_world!li22ed3:eari45ee".as_bytes().to_vec();
        match Bencode::decode_dispatch(&mut s.iter().peekable()).unwrap() {
            Bencode::Message(val) => assert_eq!(s[3..], val),
            _ => {
                assert!(false)
            }
        }
    }

    #[test]
    fn test_decode_multiple_vals_ints() {
        let s = "i33ei-1e".as_bytes().to_vec();
        let v = Bencode::decode_all(&s).unwrap();
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
        let s = "li3e5:hi55ei8e0:e".as_bytes().to_vec();
        let v = Bencode::decode_dispatch(&mut s.iter().peekable()).unwrap();
        let vec = match &v {
            Bencode::List(val) => val,
            _ => panic!("{:?}", v),
        };
        let exp = Bencode::List(vec![
            Bencode::Int(3),
            Bencode::Message("hi55e".to_owned().as_bytes().to_vec()),
            Bencode::Int(8),
            Bencode::Message("".to_owned().as_bytes().to_vec()),
        ]);
        // Checks if equality checks between lists is working
        assert_eq!(vec[0], Bencode::Int(3));
        assert_eq!(
            vec[1],
            Bencode::Message(String::from("hi55e").as_bytes().to_vec())
        );
        assert_eq!(vec[2], Bencode::Int(8));
        assert_eq!(
            vec[3],
            Bencode::Message(String::from("").as_bytes().to_vec())
        );
        assert_eq!(v, exp);
    }

    #[test]
    fn test_decode_list_no_end() {
        let s = "li44ei4e4:abcd".as_bytes().to_vec();
        assert!(Bencode::decode_dispatch(&mut s.iter().peekable()).is_err());
    }

    #[test]
    fn test_nested_list_decode() {
        let s = "li132e2:2:0:li44e0:l4:spamei22ei23ee1:fe"
            .as_bytes()
            .to_vec();
        let real = Bencode::decode_dispatch(&mut s.iter().peekable()).unwrap();
        let exp = Bencode::List(vec![
            Bencode::Int(132),
            Bencode::Message("2:".as_bytes().to_vec()),
            Bencode::Message("".as_bytes().to_vec()),
            Bencode::List(vec![
                Bencode::Int(44),
                Bencode::Message("".as_bytes().to_vec()),
                Bencode::List(vec![Bencode::Message("spam".as_bytes().to_vec())]),
                Bencode::Int(22),
                Bencode::Int(23),
            ]),
            Bencode::Message("f".as_bytes().to_owned()),
        ]);
        assert_eq!(real, exp)
    }

    #[test]
    fn test_nested_list_str() {
        let s = "llleee".as_bytes().to_vec();
        assert_eq!(
            Bencode::decode_all(&s).unwrap()[0],
            Bencode::List(vec![Bencode::List(vec![Bencode::List(
                Vec::<Bencode>::new()
            )])])
        )
    }

    #[test]
    fn test_dict_decode_pos() {
        let mut s = "d3:cow3:moo4:spam4:eggse".as_bytes().iter().peekable();
        let vals = BTreeMap::from([
            (
                "cow".as_bytes().to_vec(),
                Bencode::Message("moo".as_bytes().to_vec()),
            ),
            (
                "spam".as_bytes().to_vec(),
                Bencode::Message("eggs".as_bytes().to_vec()),
            ),
        ]);
        let x = Bencode::decode_dispatch(&mut s).unwrap();
        if let Bencode::Dict(map) = x {
            assert_eq!(map, vals);
        } else {
            assert!(false, "{:?}", x);
        }
    }

    #[test]
    fn test_nested_dict_decode() {
        let mut s = "d0:de1:adee".as_bytes().iter().peekable();
        let exp = Bencode::Dict(BTreeMap::from([
            (
                "".as_bytes().to_vec(),
                Bencode::Dict(BTreeMap::<Vec<u8>, Bencode>::new()),
            ),
            (
                "a".as_bytes().to_vec(),
                Bencode::Dict(BTreeMap::<Vec<u8>, Bencode>::new()),
            ),
        ]));
        assert_eq!(Bencode::decode_dispatch(&mut s).unwrap(), exp);
    }

    #[test]
    fn test_3_nested_dict_decode() {
        let mut s = "d2:aad3:bfgdeee".as_bytes().iter().peekable();
        let exp = Bencode::Dict(BTreeMap::from([(
            "aa".as_bytes().to_vec(),
            Bencode::Dict(BTreeMap::from([(
                "bfg".as_bytes().to_vec(),
                Bencode::Dict(BTreeMap::<Vec<u8>, Bencode>::new()),
            )])),
        )]));
        assert_eq!(Bencode::decode_dispatch(&mut s).unwrap(), exp)
    }

    #[test]
    fn test_unsorted_dict_keys_short() {
        let mut s = "d1:a2:bb1:z2:cc1:b2:gge".as_bytes().iter().peekable();
        assert!(Bencode::decode_dispatch(&mut s).is_err())
    }

    #[test]
    fn test_unsorted_dict_keys_nested() {
        let mut s = "d1:xi22e1:ad1:ai66ee1:si1ee".as_bytes().iter().peekable();
        assert!(Bencode::decode_dispatch(&mut s).is_err())
    }

    #[test]
    fn test_duplicate_keys() {
        let mut s = "d2:avi22e2:av4:bveee".as_bytes().iter().peekable();
        assert!(Bencode::decode_dispatch(&mut s).is_err())
    }
}
