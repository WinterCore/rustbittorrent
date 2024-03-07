use std::{collections::HashMap, net::SocketAddr};

use rand::Rng;

use crate::{net::udp::send_udp_packet, utils::bencode::{BencodeParser, BencodeValue}};

#[derive(Debug)]
pub struct DHTClient<'node> {
    pub node_id: [u8; 20],
    pub root_node: &'node SocketAddr,

    tx_id: u16,
}

impl<'node> DHTClient<'node> {
    pub fn new(root_node: &'node SocketAddr) -> Self {
        let node_id = rand::thread_rng().gen::<[u8; 20]>();

        Self {
            node_id,
            root_node,
            tx_id: 0,
        }
    }

    pub async fn get_peers(&mut self, infohash: &str) {
        let query = BencodeValue::Dict(
            HashMap::from([
                (
                    "t".as_bytes().to_vec(),
                    BencodeValue::Bytes(self.tx_id.to_be_bytes().to_vec()),
                ),
                (
                    "y".as_bytes().to_vec(),
                    BencodeValue::Bytes("q".as_bytes().to_vec()),
                ),
                (
                    "q".as_bytes().to_vec(),
                    BencodeValue::Bytes("get_peers".as_bytes().to_vec()),
                ),
                (
                    "a".as_bytes().to_vec(),
                    BencodeValue::Dict(HashMap::from([
                        (
                            "id".as_bytes().to_vec(),
                            BencodeValue::Bytes(self.node_id.to_vec()),
                        ),
                        (
                            "info_hash".as_bytes().to_vec(),
                            BencodeValue::Bytes(infohash.split(":").last().unwrap().as_bytes().to_owned()),
                        ),
                    ])),
                ),
            ]),
        );

        let result = send_udp_packet(self.root_node, &query.serialize()).await;
        println!("get_peers Result: {:?}", BencodeParser::new(&result.unwrap()).parse_value());
            
    }
}
