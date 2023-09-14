mod encode_tests {
    use bit_tor::bencode::Bencode;
    use std::collections::BTreeMap;
    type B = Bencode;

    #[test]
    fn encode_int_pos() {
        let to_encode = B::Int(234);
        assert_eq!(to_encode.encode_val(),"i234e");
    }

    #[test]
    fn encode_int_neg() {
        let to_encode = B::Int(-12);
        assert_eq!(to_encode.encode_val(),"i-12e")
    }

    #[test]
    fn encode_int_zero() {
        let te = B::Int(0);
        assert_eq!(te.encode_val(),"i0e")
    }

    #[test]
    fn test_message_norm(){
        let s = String::from("on_god_on_god");
        let len = s.len();
        let te = B::Message(s);
        assert_eq!(te.encode_val(),format!("{len}:{}","on_god_on_god"))
    }

    #[test]
    fn test_message_empty() {
        let te = B::Message(String::from(""));
        assert_eq!(te.encode_val(),"0:")
    }

    #[test]
    fn test_message_numeric() {
        let s = String::from("115486555");
        let l = s.len();
        let te = B::Message(s.clone());
        assert_eq!(te.encode_val(),format!("{l}:{}",s))
    }

    #[test]
    fn test_list_enc() {
        let v = vec!(
            B::Int(22),
            B::Message(String::from("spam")),
            B::Message(String::from("")),
            B::Int(0),
        );
        let te = B::List(v);
        let expected = String::from("li22e4:spam0:i0ee");
        assert_eq!(te.encode_val(),expected)
    }

    #[test]
    fn test_nested_list_enc() {
        let v = vec!(
            B::Message(String::from("CoW")),
            B::List(vec!(
                B::Int(-33),
                B::Message(String::from("eggs")),
                B::Int(234),
            )),
            B::Int(366)
        );
        let te = B::List(v);
        let expected = "l3:CoWli-33e4:eggsi234eei366ee";
        assert_eq!(te.encode_val(),expected)
    }

    #[test]
    fn test_dic_simple() {
        let dict = BTreeMap::from(
            [("tomato".to_string(),Bencode::Int(22)),
            ("".to_owned(),Bencode::Message(String::from("ferrr"))),]);
        let exp = "d0:5:ferrr6:tomatoi22ee";
        let te = B::Dict(dict);
        assert_eq!(te.encode_val(),exp)
    }

    #[test]
    fn test_dic_list() {
        let dict = BTreeMap::from([
            ("alien".to_owned(),B::Int(44)),
            ("banana".to_owned(),B::List(vec![
                B::Int(72),
                B::Message("Animal".to_owned()),
            ])),
            ("cornchip".to_owned(),B::Message("".to_owned())),
        ]);
        let exp = "d5:alieni44e6:bananali72e6:Animale8:cornchip0:e";
        let te = Bencode::Dict(dict);
        assert_eq!(te.encode_val(),exp)
    }

    #[test]
    fn test_nested_dic() {
        let inner = BTreeMap::from([
            ("Renfair".to_owned(),B::Int(-12)),
            ("goblin".to_owned(),B::Message("li23ee".to_owned()))
        ]);
        let inner = B::Dict(inner);
        let te = B::Dict(BTreeMap::from([
            ("annual".to_owned(),B::Int(111)),
            ("dict".to_owned(),inner),
            ("23".to_owned(),B::Int(22)),
            ("zebra".to_owned(),B::Message("animalhide".to_owned()))
        ]));
        let exp = "d2:23i22e6:annuali111e4:dictd7:Renfairi-12e6:goblin6:li23eee5:zebra10:animalhidee";
        assert_eq!(te.encode_val(),exp)
    }

}
