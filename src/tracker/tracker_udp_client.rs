use std::{io, net::{Ipv4Addr, SocketAddr, SocketAddrV4}};

use rand::Rng;

use crate::net::udp::send_udp_packet;


#[derive(Debug)]
pub enum TrackerUDPClientError {
    NotConnected,

    InvalidResponse,
    TransactionIdMismatch,
    Other(io::Error)
}


#[derive(Debug)]
pub struct TrackerUDPClient<'addr> {
    sock_addr: &'addr SocketAddr,
    transaction_id: i32,
    node_id: [u8; 20],

    connection_id: Option<[u8; 8]>,
}


impl<'addr> TrackerUDPClient<'addr> {
    pub fn new(sock_addr: &'addr SocketAddr) -> Self {
        let node_id = rand::thread_rng().gen::<[u8; 20]>();

        TrackerUDPClient {
            sock_addr,
            transaction_id: 0,
            node_id,
            connection_id: None,
        }
    }

    fn get_unique_transaction_id(&mut self) -> i32 {
        let value = self.transaction_id;

        self.transaction_id += 1;

        value
    }
    
    fn get_connection_id(&self) -> Result<&[u8; 8], TrackerUDPClientError> {
        if let Some(connection_id) = self.connection_id.as_ref() {
            return Ok(connection_id);
        }

        return Err(TrackerUDPClientError::NotConnected);
    }

    pub async fn connect(&mut self) -> Result<(), TrackerUDPClientError> {
        let transaction_id = self.get_unique_transaction_id();

        let mut connect_packet_data = [
            // protocol_id: Magic constant
            0x00, 0x00, 0x04, 0x17, 0x27, 0x10, 0x19, 0x80,

            // action: 0 for connect
            0x00, 0x00, 0x00, 0x00,

            // transaction_id: arbitrary number
            0x00, 0x00, 0x00, 0x00,
        ];
        
        connect_packet_data[12..16].copy_from_slice(&transaction_id.to_be_bytes());
        

        let response = send_udp_packet(&self.sock_addr, &connect_packet_data)
            .await
            .map_err(|e| TrackerUDPClientError::Other(e))?;

        
        if response.len() < 16 {
            return Err(TrackerUDPClientError::InvalidResponse);
        }

        let resp_transaction_id = i32::from_be_bytes(response[4..8].try_into().unwrap());

        if transaction_id != resp_transaction_id {
            return Err(TrackerUDPClientError::TransactionIdMismatch);
        }

        let connection_id = &response[8..16];
        self.connection_id = Some(connection_id.try_into().unwrap());

        Ok(())
    }

    pub async fn announce<'a>(&mut self, request: &AnnounceRequest<'a>) -> Result<AnnounceResponse, TrackerUDPClientError> {
        let transaction_id = self.get_unique_transaction_id();
        let connection_id = self.get_connection_id()?;

        let AnnounceRequest {
            info_hash,
            downloaded,
            left,
            uploaded,
            num_want,
            ip_address,
            port,
            event,
        } = request;

        let mut announce_packet_data = [0u8; 98];
        // connection_id
        announce_packet_data[0..8].copy_from_slice(connection_id);
        // action
        announce_packet_data[8..12].copy_from_slice(&[0x00, 0x00, 0x00, 0x01]);
        // transaction_id
        announce_packet_data[12..16].copy_from_slice(&transaction_id.to_be_bytes());
        // info_hash
        announce_packet_data[16..36].copy_from_slice(*info_hash);
        // peer_id
        announce_packet_data[36..56].copy_from_slice(&self.node_id);
        // downloaded
        announce_packet_data[56..64].copy_from_slice(&downloaded.to_be_bytes());
        // left
        announce_packet_data[64..72].copy_from_slice(&left.to_be_bytes());
        // uploaded
        announce_packet_data[72..80].copy_from_slice(&uploaded.to_be_bytes());
        // event
        announce_packet_data[80..84].copy_from_slice(&event.to_be_bytes());
        // ip_address
        announce_packet_data[84..88].copy_from_slice(ip_address);
        // key. TODO: Not sure what this is used for
        announce_packet_data[88..92].copy_from_slice(&[0x00; 4]);
        // num_wait
        announce_packet_data[92..96].copy_from_slice(&num_want.to_be_bytes());
        // port
        announce_packet_data[96..98].copy_from_slice(&port.to_be_bytes());

        let response = send_udp_packet(&self.sock_addr, &announce_packet_data)
            .await
            .map_err(|e| TrackerUDPClientError::Other(e))?;

        if response.len() < 20 {
            return Err(TrackerUDPClientError::InvalidResponse);
        }

        let announce_response = AnnounceResponse::try_from(&response[0..])
            .map_err(|_| TrackerUDPClientError::InvalidResponse)?;
        
        if announce_response.action != 1 {
            return Err(TrackerUDPClientError::InvalidResponse);
        }

        if announce_response.transaction_id != transaction_id {
            return Err(TrackerUDPClientError::TransactionIdMismatch);
        }

        Ok(announce_response)
    }
}

#[derive(Debug)]
pub struct AnnounceRequest<'hash> {
    pub info_hash: &'hash [u8; 20],
    pub downloaded: i64,
    pub left: i64,
    pub uploaded: i64,

    pub num_want: i32,

    pub ip_address: [u8; 4],
    pub port: i16,

    // 0: none; 1: completed; 2: started; 3: stopped
    pub event: i32,
}

impl<'a> Default for AnnounceRequest<'a> {
    fn default() -> Self {
        Self {
            info_hash: &[0; 20],
            downloaded: 0,
            left: 0,
            uploaded: 0,
            num_want: -1,
            ip_address: [0; 4],
            port: 6969,
            event: 0,
        }
    }
}

#[derive(Debug)]
pub struct AnnounceResponse {
    pub action: i32,
    pub transaction_id: i32,
    pub interval: i32,
    pub leechers: i32,
    pub seeders: i32,
    pub peers: Vec<SocketAddrV4>,
}

impl TryFrom<&[u8]> for AnnounceResponse {
    type Error = String;

    fn try_from(data: &[u8]) -> Result<Self, Self::Error> {
        if data.len() < 20 {
            return Err("Invalid data length".to_owned());
        }

        let action = i32::from_be_bytes(data[0..4].try_into().unwrap());
        let transaction_id = i32::from_be_bytes(data[4..8].try_into().unwrap());
        let interval = i32::from_be_bytes(data[8..12].try_into().unwrap());
        let leechers = i32::from_be_bytes(data[12..16].try_into().unwrap());
        let seeders = i32::from_be_bytes(data[16..20].try_into().unwrap());

        let peers = data[20..]
            .chunks(6)
            .map(|bytes| SocketAddrV4::new(Ipv4Addr::new(bytes[0], bytes[1], bytes[2], bytes[3]), u16::from_be_bytes([bytes[4], bytes[5]])))
            .collect::<Vec<SocketAddrV4>>();

        Ok(Self {
            action,
            transaction_id,
            interval,
            leechers,
            seeders,
            peers,
        })
    }
}
