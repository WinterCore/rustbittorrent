use std::{io, net::{Ipv4Addr, SocketAddr, SocketAddrV4, ToSocketAddrs}};

use rand::Rng;
use url::Url;
use utils::hex::decode_hex;

use net::udp::send_udp_packet;
use bittorrent::peer_client::PeerClient;

mod net;
mod utils;
mod dht_client;
mod bittorrent;
mod kademlia;
mod tracker;

#[tokio::main]
async fn main() -> io::Result<()> {
    let magnet = "magnet:?xt=urn:btih:6853ab2b86b2cb6a3c778b8aafe3dffd94242321&dn=archlinux-2024.04.01-x86_64.iso";

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

    let node_id = rand::thread_rng().gen::<[u8; 20]>();

    let infohash_bytes: [u8; 20] = decode_hex(hash.split(":").last().unwrap())
        .unwrap()
        .try_into()
        .unwrap();

    let tracker = "open.stealth.si:80".to_socket_addrs().unwrap().nth(0).unwrap();

    let connect_packet_data = [
        0x00, 0x00, 0x04, 0x17, 0x27, 0x10, 0x19, 0x80, // protocol_id: Magic constant
        0x00, 0x00, 0x00, 0x00,                         // action: 0 for connect
        0x00, 0x00, 0x00, 0x01,                         // transaction_id: arbitrary number
    ];
    
    let connect_response = send_udp_packet(&tracker, &connect_packet_data).await?;
    let connection_id = &connect_response[8..16];

    let mut announce_packet_data = [0u8; 98];
    // connection_id
    announce_packet_data[0..8].copy_from_slice(connection_id);
    // action
    announce_packet_data[8..12].copy_from_slice(&[0x00, 0x00, 0x00, 0x01]);
    // transaction_id
    announce_packet_data[12..16].copy_from_slice(&[0x00, 0x00, 0x00, 0x15]);
    // info_hash
    announce_packet_data[16..36].copy_from_slice(&infohash_bytes);
    // peer_id
    announce_packet_data[36..56].copy_from_slice(&node_id);
    // downloaded
    announce_packet_data[56..64].copy_from_slice(&[0x00; 8]);
    // left
    announce_packet_data[64..72].copy_from_slice(&[0x00; 8]);
    // uploaded
    announce_packet_data[72..80].copy_from_slice(&[0x00; 8]);
    // event
    announce_packet_data[80..84].copy_from_slice(&[0x00; 4]);
    // ip_address
    announce_packet_data[84..88].copy_from_slice(&[0x00; 4]);
    // key
    announce_packet_data[88..92].copy_from_slice(&[0x00; 4]);
    // num_wait
    announce_packet_data[92..96].copy_from_slice(&[0x00, 0x00, 0x00, 0x32]);
    // port
    announce_packet_data[96..98].copy_from_slice(&6969u16.to_be_bytes());

    let announce_response = send_udp_packet(&tracker, &announce_packet_data).await?;

    println!("Announce data: {:?}", announce_response);

    let peers = announce_response[20..]
        .chunks(6)
        .map(|bytes| SocketAddr::V4(SocketAddrV4::new(Ipv4Addr::new(bytes[0], bytes[1], bytes[2], bytes[3]), u16::from_be_bytes([bytes[4], bytes[5]]))))
        .collect::<Vec<SocketAddr>>();

    println!("---------------------------------------");
    for peer_addr in peers {
        println!("Trying to connect to {:?}", peer_addr);

        let peer_connection = PeerClient::connect(&node_id, &peer_addr, &infohash_bytes).await;

        if let Err(err) = peer_connection {
            println!("Error connecting to {:?}: {:?}", peer_addr, err);

            continue;
        }

        println!("Connected successfully to {:?}: {:?}", peer_addr, peer_connection);

        let handshake_result = peer_connection.unwrap().send_handshake().await;

        println!("Handshake result {:?}: {:?}", peer_addr, handshake_result);

        break;
    }


    Ok(())
}
