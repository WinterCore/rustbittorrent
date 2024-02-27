use std::collections::HashMap;
use std::fs;
use std::str;

use net::udp::send_udp_packet;
use rand::distributions::Alphanumeric;
use rand::Rng;
use rand::RngCore;
use url::Url;
use utils::bencode::BencodeParser;

use utils::bencode::BencodeValue;
use tracker::Tracker;

mod net;
mod tracker;
mod utils;

#[tokio::main]
async fn main() {
    let magnet = "magnet:?xt=urn:btih:dd8255ecdc7ca55fb0bbf81323d87062db1f6d1c&dn=Big+Buck+Bunny&tr=udp%3A%2F%2Fexplodie.org%3A6969&tr=udp%3A%2F%2Ftracker.coppersurfer.tk%3A6969&tr=udp%3A%2F%2Ftracker.empire-js.us%3A1337&tr=udp%3A%2F%2Ftracker.leechers-paradise.org%3A6969&tr=udp%3A%2F%2Ftracker.opentrackr.org%3A1337&tr=wss%3A%2F%2Ftracker.btorrent.xyz&tr=wss%3A%2F%2Ftracker.fastcast.nz&tr=wss%3A%2F%2Ftracker.openwebtorrent.com&ws=https%3A%2F%2Fwebtorrent.io%2Ftorrents%2F&xs=https%3A%2F%2Fwebtorrent.io%2Ftorrents%2Fbig-buck-bunny.torrent";


    let url = Url::parse(magnet).expect("Should parse magnet link");
    let query_pairs: Vec<(String, String)> = url
        .query_pairs()
        .map(|(k, v)| (k.to_string(), v.to_string()))
        .collect();

    // We only care about udp trackers for now
    let tracker_url = query_pairs
        .iter()
        .find(|(k, v)| k == "tr" && v.starts_with("udp://"))
        .and_then(|x| Url::parse(&x.1).ok())
        .expect("Magnet link should have at least 1 tracker");
    let node_id: Vec<u8> = rand::thread_rng()
        .sample_iter(&Alphanumeric)
        .take(20)
        .collect();

    println!("URL: {:?}", query_pairs);
    
    // Ping 
    let buff = BencodeValue::Dict(
        HashMap::from([
            (
                "id".as_bytes().to_vec(),
                BencodeValue::Bytes(node_id.to_vec()),
            ),
            (
                "q".as_bytes().to_vec(),
                BencodeValue::Bytes("ping".as_bytes().to_vec()),
            ),
        ]),
    );

    let resp = send_udp_packet(
        &tracker_url,
        &buff.serialize(),
    ).await;

    println!("{:?}", resp);

    // println!("Query pairs: {:?}", tracker);

    /*
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
    */
}
