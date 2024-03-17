use std::{collections::HashMap, io, net::{Ipv4Addr, SocketAddr, SocketAddrV4}, str};

use rand::Rng;

use crate::{net::udp::send_udp_packet, utils::bencode::{BencodeParser, BencodeValue}};

#[derive(Debug)]
pub struct CompactNodeInfo {
    pub node_id: [u8; 20],
    pub socket_addr: SocketAddrV4,
}

#[derive(Debug)]
pub enum DHTErrorCode {
    GenericError = 201,
    ServerError = 202,
    ProtocolError = 203,
    MethodUnknown = 204,
}

impl TryFrom<i64> for DHTErrorCode {
    type Error = String;

    fn try_from(value: i64) -> Result<Self, Self::Error> {
        match value {
            201 => Ok(Self::GenericError),
            202 => Ok(Self::ServerError),
            203 => Ok(Self::ProtocolError),
            204 => Ok(Self::MethodUnknown),
            _ => Err(format!("Unknown DHT error code {}", value))
        }
    }
}


#[derive(Debug)]
pub struct DHTError {
    pub error_code: DHTErrorCode,
    pub error_message: String,
}

#[derive(Debug)]
pub struct DHTGetPeersData {
    /// The token (used for announce_peer). It seems to have a size of 4 bytes but I'm not entirely
    /// sure
    pub token: Option<Vec<u8>>,

    /// [node_id(20 bytes), ip(4 bytes), port(2 bytes), ...]
    pub nodes: Vec<CompactNodeInfo>,
}

#[derive(Debug)]
pub enum DHTResponseData {
    GetPeersData(DHTGetPeersData),
    DHTError(DHTError),
}

#[derive(Debug)]
pub struct DHTResponse {
    /// Not sure what this is. I assume it's the ip of the replying node but it's useless
    /// since we already have it
    pub socket_addr: SocketAddrV4,

    /// The node id of the replying node
    pub node_id: [u8; 20],

    pub data: DHTResponseData,
}

impl TryFrom<BencodeValue> for DHTResponse {
    type Error = String;

    fn try_from(value: BencodeValue) -> Result<Self, Self::Error> {
        let dict = value.dict().map_err(|_| "BencodeValue should be an object")?;
        let ip_bytes = dict
            .get("ip".as_bytes())
            .and_then(|bv| bv.bytes().ok())
            .ok_or("Failed to parse id")?;
        
        let socket_addr = SocketAddrV4::new(
            Ipv4Addr::new(ip_bytes[0], ip_bytes[1], ip_bytes[2], ip_bytes[3]),
            u16::from_be_bytes([ip_bytes[4], ip_bytes[5]]),
        );

        let y = dict
            .get("y".as_bytes())
            .and_then(|bv| bv.bytes().ok())
            .ok_or("Failed to parse response type")?;

        let r = dict
            .get("r".as_bytes())
            .and_then(|bv| bv.dict().ok())
            .ok_or("Failed to parse response dict")?;

        let node_id: [u8; 20] = r
            .get("id".as_bytes())
            .and_then(|x| x.bytes().ok())
            .and_then(|x| x.clone().try_into().ok())
            .ok_or("Failed to parse node_id")?;

        if y[0] == 'e' as u8 {
            let e = r
                .get("e".as_bytes())
                .and_then(|x| x.list().ok())
                .ok_or("Failed to parse response error")?;

            let error_code = e
                .get(0)
                .and_then(|x| x.integer().ok())
                .ok_or("Failed to parse response error code")?;

            let error_message = e
                .get(1)
                .and_then(|x| x.bytes().ok())
                .ok_or("Failed to parse response error message")?;

            let error = DHTError {
                error_code: DHTErrorCode::try_from(*error_code)?,
                error_message: String::from_utf8_lossy(error_message).into_owned(),
            };

            return Ok(Self {
                socket_addr,
                node_id,
                data: DHTResponseData::DHTError(error)
            })
        }

        unimplemented!();
        /*
        Ok(Self {
             
        })
        */
    }
}

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

    pub async fn get_peers(&mut self, infohash: &[u8]) -> io::Result<BencodeValue> {
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
                            BencodeValue::Bytes(infohash.to_owned()),
                        ),
                    ])),
                ),
            ]),
        );

        let resp = send_udp_packet(self.root_node, &query.serialize()).await?;


        let value = BencodeParser::new(&resp)
            .parse_value()
            .map_err(|_| io::Error::new(io::ErrorKind::InvalidData, "Failed to parse bencode response"))?;

        Ok(value)
    }
}
