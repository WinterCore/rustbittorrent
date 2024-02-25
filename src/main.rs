use std::fs;
use std::str;

use url::Url;
use utils::bencode::BencodeParser;

use utils::bencode::BencodeValue;
use tracker::Tracker;

mod net;
mod tracker;
mod utils;

#[tokio::main]
async fn main() {
    let torrent_file_data = fs::read("./test-torrent.torrent")
        .expect("Should read file");

    let data = BencodeParser::new(&torrent_file_data)
        .parse_value()
        .expect("Should parse torrent file");

    let torrent_data = match data {
        BencodeValue::Dict(map) => map,
        _ => panic!("Torrent data should be a dict"),
    };

    let url = match torrent_data.get("announce".as_bytes()).expect("announce prop should exist") {
        BencodeValue::Bytes(bytes) => {
            let urlstr = str::from_utf8(bytes).expect("announce should contain a valid string");
            Url::parse(urlstr).expect("URL should be valid")
        },
        _ => panic!("Invalid type for announce data"),
    };
    
    println!("Debug: {}", url);
    let tracker_client = Tracker::new(&url);
    let connect_result = tracker_client.connect().await.expect("Should connect");
    
    println!("URL: {:?}", url);
}
