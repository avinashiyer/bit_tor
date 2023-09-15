use bit_tor::bencode::Bencode;


fn main() {
    let s = "d4:infod9:file treed4:dir1d4:dir2d9:fileA.txtd0:d6:lengthi1024e11:pieces root32:aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaeeeeeee";
    println!("{:#?}",Bencode::decode_all(s)[0]);
    println!("Hello, world!");
}
