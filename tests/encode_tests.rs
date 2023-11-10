mod encode_tests {
    use bit_tor::bencode::Bencode;
    use std::collections::BTreeMap;

    #[test]
    fn encode_int_pos() {
        let to_encode = Bencode::Int(234);
        let exp = [105u8, 50, 51, 52, 101];
        assert_eq!(to_encode.encode_val(), exp);
    }

    #[test]
    fn encode_int_neg() {
        let to_encode = Bencode::Int(-12);
        let exp = [105u8, 45, 49, 50, 101];
        assert_eq!(to_encode.encode_val(), exp)
    }

    #[test]
    fn encode_int_zero() {
        let te = Bencode::Int(0);
        assert_eq!(te.encode_val(), [105u8, 48, 101])
    }

    #[test]
    fn test_message_norm() {
        let s = "on_god_on_god".as_bytes().to_vec();
        let te = Bencode::Message(s);
        let exp = [
            49u8, 51, 58, 111, 110, 95, 103, 111, 100, 95, 111, 110, 95, 103, 111, 100,
        ];
        assert_eq!(te.encode_val(), exp)
    }

    #[test]
    fn test_message_empty() {
        let te = Bencode::Message("".as_bytes().to_vec());
        assert_eq!(te.encode_val(), [48u8, 58])
    }

    #[test]
    fn test_message_numeric() {
        let mut s = "115486555".as_bytes().to_vec();
        let mut l = s.len().to_string().as_bytes().to_vec();
        l.push(b':');
        let te = Bencode::Message(s.clone());
        l.append(&mut s);
        assert_eq!(te.encode_val(), l)
    }

    #[test]
    fn test_list_enc() {
        let v = vec![
            Bencode::Int(22),
            Bencode::Message("spam".as_bytes().to_vec()),
            Bencode::Message("".as_bytes().to_vec()),
            Bencode::Int(0),
        ];
        let te = Bencode::List(v);
        let expected = "li22e4:spam0:i0ee".as_bytes().to_vec();
        assert_eq!(te.encode_val(), expected)
    }

    #[test]
    fn test_nested_list_enc() {
        let v = vec![
            Bencode::Message("CoW".as_bytes().to_vec()),
            Bencode::List(vec![
                Bencode::Int(-33),
                Bencode::Message("eggs".as_bytes().to_vec()),
                Bencode::Int(234),
            ]),
            Bencode::Int(366),
        ];
        let te = Bencode::List(v);
        let expected = "l3:CoWli-33e4:eggsi234eei366ee".as_bytes().to_vec();
        assert_eq!(te.encode_val(), expected)
    }

    #[test]
    fn test_dic_simple() {
        let dict = BTreeMap::from([
            ("tomato".as_bytes().to_vec(), Bencode::Int(22)),
            (
                "".as_bytes().to_vec(),
                Bencode::Message("ferrr".as_bytes().to_vec()),
            ),
        ]);
        let exp = "d0:5:ferrr6:tomatoi22ee".as_bytes().to_vec();
        let te = Bencode::Dict(dict);
        assert_eq!(te.encode_val(), exp)
    }

    #[test]
    fn test_dic_list() {
        let dict = BTreeMap::from([
            ("alien".as_bytes().to_vec(), Bencode::Int(44)),
            (
                "banana".as_bytes().to_vec(),
                Bencode::List(vec![
                    Bencode::Int(72),
                    Bencode::Message("Animal".as_bytes().to_vec()),
                ]),
            ),
            (
                "cornchip".as_bytes().to_vec(),
                Bencode::Message("".as_bytes().to_vec()),
            ),
        ]);
        let exp = "d5:alieni44e6:bananali72e6:Animale8:cornchip0:e"
            .as_bytes()
            .to_vec();
        let te = Bencode::Dict(dict);
        assert_eq!(te.encode_val(), exp)
    }

    #[test]
    fn test_nested_dic() {
        let inner = BTreeMap::from([
            ("Renfair".as_bytes().to_vec(), Bencode::Int(-12)),
            (
                "goblin".as_bytes().to_vec(),
                Bencode::Message("li23ee".as_bytes().to_vec()),
            ),
        ]);
        let inner = Bencode::Dict(inner);
        let te = Bencode::Dict(BTreeMap::from([
            ("annual".as_bytes().to_vec(), Bencode::Int(111)),
            ("dict".as_bytes().to_vec(), inner),
            ("23".as_bytes().to_vec(), Bencode::Int(22)),
            (
                "zebra".as_bytes().to_vec(),
                Bencode::Message("animalhide".as_bytes().to_vec()),
            ),
        ]));
        let exp =
            "d2:23i22e6:annuali111e4:dictd7:Renfairi-12e6:goblin6:li23eee5:zebra10:animalhidee"
                .as_bytes()
                .to_vec();
        assert_eq!(te.encode_val(), exp)
    }
}
