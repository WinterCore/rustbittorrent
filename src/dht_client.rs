use std::{collections::HashMap, io, net::{Ipv4Addr, SocketAddr, SocketAddrV4}, str, time::Duration};

use rand::Rng;
use tokio::time::timeout;

use crate::{net::udp::send_udp_packet, utils::bencode::{BencodeParser, BencodeValue}};

#[derive(Debug, Clone, PartialEq)]
pub struct CompactNodeInfo {
    pub node_id: [u8; 20],
    pub socket_addr: SocketAddrV4,
}

impl From<&[u8]> for CompactNodeInfo {
    fn from(bytes: &[u8]) -> Self {
        let socket_addr = SocketAddrV4::new(
            Ipv4Addr::new(bytes[20], bytes[21], bytes[22], bytes[23]),
            u16::from_be_bytes([bytes[24], bytes[25]]),
        );

        Self {
            // TODO: Maybe it's not wise to use unwrap here
            node_id: bytes[0..20].try_into().unwrap(),
            socket_addr,
        }
    }
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
pub struct DHTBaseResponse {
    /*
    /// Not sure what this is. I assume it's the ip of the replying node but it's useless
    /// since we already have it
    pub socket_addr: SocketAddrV4,
    */

    /// The node id of the replying node
    pub node_id: [u8; 20],
}

impl TryFrom<&BencodeValue> for DHTBaseResponse {
    type Error = String;

    fn try_from(value: &BencodeValue) -> Result<Self, Self::Error> {
        let r = get_resp_dict(value)?;

        let node_id: [u8; 20] = r
            .get("id".as_bytes())
            .and_then(|x| x.bytes().ok())
            .and_then(|x| x.clone().try_into().ok())
            .ok_or("Failed to parse node_id")?;

        Ok(Self { node_id })
    }
}

#[derive(Debug)]
enum DHTResponseType {
    Response,
    Error,
}

impl DHTBaseResponse {
}

#[derive(Debug)]
pub struct DHTGetPeersResponse {
    pub base: DHTBaseResponse,

    /// The token (used for announce_peer). It seems to have a size of 4 bytes but I'm not entirely
    /// sure
    pub token: Option<Vec<u8>>,

    /// Nodes that we can contact asking for an infohash
    /// [node_id(20 bytes), ip(4 bytes), port(2 bytes), ...]
    pub nodes: Vec<CompactNodeInfo>,

    /// Peers for the provided infohash (nodes that have the torrent?)
    pub values: Vec<SocketAddrV4>,
}

impl TryFrom<&BencodeValue> for DHTGetPeersResponse {
    type Error = String;

    fn try_from(value: &BencodeValue) -> Result<Self, Self::Error> {
        let base = DHTBaseResponse::try_from(value)?;

        let r = get_resp_dict(&value)?;

        let token = r
            .get("token".as_bytes())
            .and_then(|x| x.bytes().ok())
            .cloned();

        let nodes = r
            .get("nodes".as_bytes())
            .and_then(|x| x.bytes().ok())
            .ok_or("Failed to parse response error message")?
            .chunks(26)
            .map(|bytes| CompactNodeInfo::from(bytes))
            .collect();

        let values = r
            .get("values".as_bytes())
            .and_then(|x| x.bytes().ok())
            .map(|bytes| {
                bytes
                    .chunks(6)
                    .map(|bs| {
                        SocketAddrV4::new(
                            Ipv4Addr::new(bs[0], bs[1], bs[2], bs[3]),
                            u16::from_be_bytes([bs[4], bs[5]])
                        )
                    })
                    .collect()
            }).unwrap_or(vec![]);

        let resp = Self {
            base,
            token,
            nodes,
            values,
        };

        return Ok(resp)
    }
}

#[derive(Debug)]
pub struct DHTFindNodeResponse {
    pub base: DHTBaseResponse,

    pub nodes: Vec<CompactNodeInfo>,
}

impl TryFrom<&BencodeValue> for DHTFindNodeResponse {
    type Error = String;

    fn try_from(value: &BencodeValue) -> Result<Self, Self::Error> {
        let base = DHTBaseResponse::try_from(value)?;

        let r = get_resp_dict(&value)?;

        let nodes = r
            .get("nodes".as_bytes())
            .and_then(|x| x.bytes().ok())
            .ok_or("Failed to parse response error message")?
            .chunks(26)
            .map(|bytes| CompactNodeInfo::from(bytes))
            .collect();

        let resp = Self {
            base,
            nodes,
        };

        return Ok(resp)
    }
}

#[derive(Debug)]
pub struct DHTErrorResponse {
    pub base: DHTBaseResponse,

    pub error_code: DHTErrorCode,
    pub error_message: String,
}

impl TryFrom<&BencodeValue> for DHTErrorResponse {
    type Error = String;

    fn try_from(value: &BencodeValue) -> Result<Self, Self::Error> {
        let base = DHTBaseResponse::try_from(value)?;
        let root_dict = value.dict().map_err(|_| "BencodeValue should be an object")?;

        let e = root_dict
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

        let error_resp = DHTErrorResponse {
            base,
            error_code: DHTErrorCode::try_from(*error_code)?,
            error_message: String::from_utf8_lossy(error_message).into_owned(),
        };

        return Ok(error_resp)

    }
}

fn get_resp_dict(value: &BencodeValue) -> Result<&HashMap<Vec<u8>, BencodeValue>, String> {
    let root_dict = value.dict().map_err(|_| "BencodeValue should be an object")?;

    let r = root_dict
        .get("r".as_bytes())
        .and_then(|bv| bv.dict().ok())
        .ok_or("Failed to parse response dict")?;

    Ok(r)
}

fn get_response_type(value: &BencodeValue) -> Result<DHTResponseType, String> {
    let root_dict = value.dict().map_err(|_| "BencodeValue should be an object")?;
    
    let y = root_dict
        .get("y".as_bytes())
        .and_then(|bv| bv.bytes().ok())
        .ok_or("Failed to parse response type")?;

    match y[0] as char {
        'e' => Ok(DHTResponseType::Error),
        'r' => Ok(DHTResponseType::Response),
        _ => Err("Unsupported DHT response type".to_owned()),
    }
}

#[derive(Debug)]
pub enum DHTResponse<T> {
    DHTError(DHTErrorResponse),
    DHTResponse(T),
}

impl<T> DHTResponse<T> {
    pub fn unwrap(self) -> T {
        match self {
            DHTResponse::DHTResponse(data) => data,
            _ => panic!("Unwrap DHTRespose failed"),
        }
    }
}

impl<'a, T: TryFrom<&'a BencodeValue, Error = String>> TryFrom<&'a BencodeValue> for DHTResponse<T> {
    type Error = String;

    fn try_from(value: &'a BencodeValue) -> Result<Self, Self::Error> {
        let response_type = get_response_type(&value)?;
        
        match response_type {
            DHTResponseType::Response => Ok(DHTResponse::DHTResponse(T::try_from(value)?)),
            DHTResponseType::Error => Ok(DHTResponse::DHTError(DHTErrorResponse::try_from(value)?)),
        }
    }
}

#[derive(Debug)]
pub struct DHTClient<'nodeId, 'node> {
    pub node_id: &'nodeId [u8; 20],
    pub root_node: &'node SocketAddr,

    tx_id: u16,
}

impl<'node, 'nodeId> DHTClient<'nodeId, 'node> {
    pub fn new(node_id: &'nodeId [u8; 20], root_node: &'node SocketAddr) -> Self {

        Self {
            node_id,
            root_node,
            tx_id: 0,
        }
    }

    pub async fn get_peers(&self, infohash: &[u8; 20]) -> Result<DHTResponse<DHTGetPeersResponse>, String> {
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
                            BencodeValue::Bytes(infohash.to_vec()),
                        ),
                    ])),
                ),
            ]),
        );

        let resp = timeout(
            Duration::from_secs(5),
            send_udp_packet(self.root_node, &query.serialize()),
        ).await
        .map_err(|x| format!("Timeout reached {}", x.to_string()))?
        .map_err(|x| format!("Failed to send udp packet {}", x.to_string()))?;

        let value = BencodeParser::new(&resp)
            .parse_value()
            .map_err(|_| "Failed to parse bencode response")?;

        let response = DHTResponse::<DHTGetPeersResponse>::try_from(&value)?;
        
        Ok(response)
    }

    pub async fn find_node(&self, target: &[u8]) -> Result<DHTResponse<DHTFindNodeResponse>, String> {
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
                    BencodeValue::Bytes("find_node".as_bytes().to_vec()),
                ),
                (
                    "a".as_bytes().to_vec(),
                    BencodeValue::Dict(HashMap::from([
                        (
                            "id".as_bytes().to_vec(),
                            BencodeValue::Bytes(self.node_id.to_vec()),
                        ),
                        (
                            "target".as_bytes().to_vec(),
                            BencodeValue::Bytes(target.to_owned()),
                        ),
                    ])),
                ),
            ]),
        );

        let resp = timeout(
            Duration::from_secs(3),
            send_udp_packet(self.root_node, &query.serialize())
        ).await
        .map_err(|x| format!("Timeout reached {}", x.to_string()))?
        .map_err(|x| format!("Failed to send udp packet {}", x.to_string()))?;

        let value = BencodeParser::new(&resp)
            .parse_value()
            .map_err(|_| "Failed to parse bencode response")?;

        let response = DHTResponse::<DHTFindNodeResponse>::try_from(&value)?;
        
        Ok(response)
    }
}
