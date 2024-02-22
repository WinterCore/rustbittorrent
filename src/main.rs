use std::fs;

use utils::bencode::BencodeParser;

use crate::utils::bencode::BencodeValue;

mod utils;

fn main() {
    let torrent_file_data = fs::read("./puppy.torrent")
        .expect("Should read file");

    let data = BencodeParser::new(&torrent_file_data)
        .parse_value()
        .expect("Should parse torrent file");

    // println!("{:?}", data);
}
