mod encode_tests {
    use bit_tor::bencode::Bencode;
    use std::collections::BTreeMap;

    #[test]
    fn encode_int_pos() {
        let to_encode = Bencode::Int(234);
        assert_eq!(to_encode.encode_val(),"i234e");
    }

    #[test]
    fn encode_int_neg() {
        let to_encode = Bencode::Int(-12);
        assert_eq!(to_encode.encode_val(),"i-12e")
    }

    #[test]
    fn encode_int_zero() {
        let te = Bencode::Int(0);
        assert_eq!(te.encode_val(),"i0e")
    }

    #[test]
    fn test_message_norm(){
        let s = "on_god_on_god".as_bytes().to_vec();
        let len = s.len();
        let te = Bencode::Message(s);
        assert_eq!(te.encode_val(),format!("{len}:{}","on_god_on_god"))
    }

    #[test]
    fn test_message_empty() {
        let te = Bencode::Message("".as_bytes().to_vec());
        assert_eq!(te.encode_val(),"0:")
    }

    #[test]
    fn test_message_numeric() {
        let s = "115486555".as_bytes().to_vec();
        let l = s.len();
        let te = Bencode::Message(s.clone());
        assert_eq!(te.encode_val(),format!("{l}:{}",String::from_utf8(s).expect("could not convert from to utf8")))
    }

    #[test]
    fn test_list_enc() {
        let v = vec!(
            Bencode::Int(22),
            Bencode::Message("spam".as_bytes().to_vec()),
            Bencode::Message("".as_bytes().to_vec()),
            Bencode::Int(0),
        );
        let te = Bencode::List(v);
        let expected = String::from("li22e4:spam0:i0ee");
        assert_eq!(te.encode_val(),expected)
    }

    #[test]
    fn test_nested_list_enc() {
        let v = vec!(
            Bencode::Message("CoW".as_bytes().to_vec()),
            Bencode::List(vec!(
                Bencode::Int(-33),
                Bencode::Message("eggs".as_bytes().to_vec()),
                Bencode::Int(234),
            )),
            Bencode::Int(366)
        );
        let te = Bencode::List(v);
        let expected = "l3:CoWli-33e4:eggsi234eei366ee";
        assert_eq!(te.encode_val(),expected)
    }

    #[test]
    fn test_dic_simple() {
        let dict = BTreeMap::from(
            [("tomato".as_bytes().to_vec(),Bencode::Int(22)),
            ("".as_bytes().to_vec(),Bencode::Message("ferrr".as_bytes().to_vec())),]);
        let exp = "d0:5:ferrr6:tomatoi22ee";
        let te = Bencode::Dict(dict);
        assert_eq!(te.encode_val(),exp)
    }

    #[test]
    fn test_dic_list() {
        let dict = BTreeMap::from([
            ("alien".as_bytes().to_vec(),Bencode::Int(44)),
            ("banana".as_bytes().to_vec(),Bencode::List(vec![
                Bencode::Int(72),
                Bencode::Message("Animal".as_bytes().to_vec())
            ])),
            ("cornchip".as_bytes().to_vec(),Bencode::Message("".as_bytes().to_vec())),
        ]);
        let exp = "d5:alieni44e6:bananali72e6:Animale8:cornchip0:e";
        let te = Bencode::Dict(dict);
        assert_eq!(te.encode_val(),exp)
    }

    #[test]
    fn test_nested_dic() {
        let inner = BTreeMap::from([
            ("Renfair".as_bytes().to_vec(),Bencode::Int(-12)),
            ("goblin".as_bytes().to_vec(),Bencode::Message("li23ee".as_bytes().to_vec()))
        ]);
        let inner = Bencode::Dict(inner);
        let te = Bencode::Dict(BTreeMap::from([
            ("annual".as_bytes().to_vec(),Bencode::Int(111)),
            ("dict".as_bytes().to_vec(),inner),
            ("23".as_bytes().to_vec(),Bencode::Int(22)),
            ("zebra".as_bytes().to_vec(),Bencode::Message("animalhide".as_bytes().to_vec()))
        ]));
        let exp = "d2:23i22e6:annuali111e4:dictd7:Renfairi-12e6:goblin6:li23eee5:zebra10:animalhidee";
        assert_eq!(te.encode_val(),exp)
    }

}
