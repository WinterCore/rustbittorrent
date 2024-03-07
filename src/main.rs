use std::collections::HashMap;
use std::fs;
use std::net::IpAddr;
use std::net::Ipv4Addr;
use std::net::SocketAddr;
use std::str;

use dht_client::DHTClient;
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
mod dht_client;

#[tokio::main]
async fn main() {
    let magnet = "magnet:?xt=urn:btih:dd8255ecdc7ca55fb0bbf81323d87062db1f6d1c&dn=Big+Buck+Bunny&tr=udp%3A%2F%2Fexplodie.org%3A6969&tr=udp%3A%2F%2Ftracker.coppersurfer.tk%3A6969&tr=udp%3A%2F%2Ftracker.empire-js.us%3A1337&tr=udp%3A%2F%2Ftracker.leechers-paradise.org%3A6969&tr=udp%3A%2F%2Ftracker.opentrackr.org%3A1337&tr=wss%3A%2F%2Ftracker.btorrent.xyz&tr=wss%3A%2F%2Ftracker.fastcast.nz&tr=wss%3A%2F%2Ftracker.openwebtorrent.com&ws=https%3A%2F%2Fwebtorrent.io%2Ftorrents%2F&xs=https%3A%2F%2Fwebtorrent.io%2Ftorrents%2Fbig-buck-bunny.torrent";


    let url = Url::parse(magnet).expect("Should parse magnet link");
    let query_pairs: Vec<(String, String)> = url
        .query_pairs()
        .map(|(k, v)| (k.to_string(), v.to_string()))
        .collect();

    let hash = query_pairs
        .iter()
        .find(|x| x.0 == "xt")
        .map(|x| &x.1)
        .expect("Magnet link should contain infohash");

    let root_node = SocketAddr::new(IpAddr::V4(Ipv4Addr::new(67, 215, 246, 10)), 6881);
    
    let mut dht_client = DHTClient::new(&root_node);

    dht_client.get_peers(hash).await;

    /*
    let infohash = query_pairs
        .iter()
        .filter_map(|x| {
            if x.0 ==
        })
        */

    /*
    // We only care about udp trackers for now
    let tracker_urls: Vec<_> = query_pairs
        .iter()
        .filter(|(k, v)| k == "xs" && v.starts_with("http"))
        .collect();
    let node_id: [u8; 20] = rand::thread_rng().gen::<[u8; 20]>();

    // Ping 
    let buff = BencodeValue::Dict(
        HashMap::from([
            (
                "t".as_bytes().to_vec(),
                BencodeValue::Bytes([0x00, 0x01].to_vec()),
            ),
            (
                "y".as_bytes().to_vec(),
                BencodeValue::Bytes("q".as_bytes().to_vec()),
            ),
            (
                "q".as_bytes().to_vec(),
                BencodeValue::Bytes("ping".as_bytes().to_vec()),
            ),
            (
                "a".as_bytes().to_vec(),
                BencodeValue::Dict(HashMap::from([
                    (
                        "id".as_bytes().to_vec(),
                        BencodeValue::Bytes(node_id.to_vec()),
                    ),
                    /*
                    (
                        "target".as_bytes().to_vec(),
                        BencodeValue::Bytes([0x58, 0x77, 0x07, 0x8a, 0x79, 0x0c, 0xaa, 0xe9, 0x50, 0xee, 0x0f, 0x0f, 0x38, 0xba, 0xca, 0x49, 0x78, 0x64, 0x2b, 0xb9].to_vec()),
                    ),
                    */
                ])),
            ),
        ]),
    );

    // DHT Initialize node list
    // router.utorrent.com
    // router.bittorrent.com
    // dht.transmissionbt.com

    let custom_url = Url::parse("udp://router.bittorrent.com:6881").unwrap();

    println!("Tracker urls: {:?}", tracker_urls);

    let resp = send_udp_packet(
        &custom_url,
        &buff.serialize(),
    ).await;

    println!("{:#?}", BencodeParser::new(&resp.unwrap()).parse_value().unwrap());

    // println!("Query pairs: {:?}", tracker);

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
