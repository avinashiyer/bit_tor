mod bencode_tests {
    use bit_tor::bencode::BencodeVal;
    use std::collections::HashMap;
    #[test]
    fn decode_pos_int() {
        let num = 98;
        let s = format!("i{num}e");
        let vals = BencodeVal::decode_all(&s);
        if vals.len() != 1 {
            panic!(
                "Returned Wrong amount of values. \nExpected: 1 Actual: {}.",
                vals.len()
            )
        }
        match vals[0] {
            BencodeVal::Int(i) => {
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
        BencodeVal::decode_single(&mut s.char_indices().peekable());
    }

    #[test]
    fn neg_int_decode() {
        let num = -32;
        let s = format!("i{num}e");
        match BencodeVal::decode_single(&mut s.char_indices().peekable()) {
            BencodeVal::Int(n) => assert_eq!(n, num),
            _ => {
                assert!(false)
            }
        }
    }

    #[test]
    fn test_message_decode() {
        let s = String::from("12:Hello World!");
        let res = BencodeVal::decode_single(&mut s.char_indices().peekable());
        match res {
            BencodeVal::Message(x) => {
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
        BencodeVal::decode_single(&mut s.char_indices().peekable());
    }

    #[test]
    fn test_message_with_bencoded_vals() {
        let s = "35:abcd12:hello_world!li22ed3:eari45ee";
        match BencodeVal::decode_single(&mut s.char_indices().peekable()) {
            BencodeVal::Message(val) => assert_eq!(s[3..], val),
            _ => {
                assert!(false)
            }
        }
    }

    #[test]
    fn test_decode_multiple_vals_ints() {
        let s = "i33ei-1e";
        let v = BencodeVal::decode_all(s);
        println!("{:?}", v);
        match v[0] {
            BencodeVal::Int(x) => assert_eq!(33, x),
            _ => {
                assert!(false)
            }
        }
        match v[1] {
            BencodeVal::Int(x) => assert_eq!(-1, x),
            _ => assert!(false),
        }
    }

    #[test]
    fn test_decode_list_pos() {
        let mut s = "li3e5:hi55ei8e0:e".char_indices().peekable();
        let v = BencodeVal::decode_single(&mut s);
        let vec = match v {
            BencodeVal::List(val) => val,
            _ => panic!("{:?}", v),
        };
        assert_eq!(vec[0], BencodeVal::Int(3));
        assert_eq!(vec[1], BencodeVal::Message(String::from("hi55e")));
        assert_eq!(vec[2], BencodeVal::Int(8));
        assert_eq!(vec[3], BencodeVal::Message(String::from("")));
    }

    #[test]
    #[should_panic]
    fn test_decode_list_no_end() {
        let mut s = "li44ei4e4:abcd".char_indices().peekable();
        BencodeVal::decode_single(&mut s);
    }
    // TODO: Nested List

    #[test]
    fn test_dict_decode_pos() {
        let mut s = "d3:cow3:moo4:spam4:eggse".char_indices().peekable();
        let vals = HashMap::from([
            (
                String::from("cow"),
                BencodeVal::Message(String::from("moo")),
            ),
            (
                String::from("spam"),
                BencodeVal::Message(String::from("eggs")),
            ),
        ]);
        let x = BencodeVal::decode_single(&mut s);
        if let BencodeVal::Dict(map) = x {
            assert_eq!(map, vals);
        } else {
            assert!(false,"{:?}",x);
        }
    }
}
