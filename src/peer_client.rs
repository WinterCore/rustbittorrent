use std::{io, net::{Ipv4Addr, SocketAddrV4}};

use tokio::{io::{AsyncReadExt, AsyncWriteExt}, net::TcpStream};



#[derive(Debug)]
pub struct PeerClient<'addr> {
    pub socket_addr: &'addr SocketAddrV4,
    pub infohash: [u8; 20],
    pub node_id: [u8; 20],
    
    stream: TcpStream,
}

impl<'addr> PeerClient<'addr> {
    pub async fn connect(
        socket_addr: &'addr SocketAddrV4,
        infohash: &[u8; 20],
        node_id: &[u8; 20],
    ) -> io::Result<Self> {
        let stream = TcpStream::connect(socket_addr).await?;

        Ok(Self {
            infohash: infohash.clone(),
            node_id: node_id.clone(),
            socket_addr,
            stream,
        })
    }

    pub async fn send_handshake(&mut self) -> io::Result<()> {
        let mut handshake_data: Vec<u8> = vec![];

        handshake_data.push(19);
        handshake_data.extend_from_slice("BitTorrent protocol".as_bytes());
        handshake_data.extend_from_slice(&[0u8; 8]);
        handshake_data.extend_from_slice(&self.infohash);
        handshake_data.extend_from_slice(&self.node_id);

        self.stream.write_all(&handshake_data).await?;

        let mut buf = [0u8; 68];
        let resp = self.stream.read_exact(&mut buf).await?;
        println!("Received TCP resp: {:?} {:?}", buf, resp);
        /*
         *
         * Handshake
The handshake is a required message and must be the first message transmitted by the client. It is (49+len(pstr)) bytes long.

handshake: <pstrlen><pstr><reserved><info_hash><peer_id>

pstrlen: string length of <pstr>, as a single raw byte
pstr: string identifier of the protocol
reserved: eight (8) reserved bytes. All current implementations use all zeroes. Each bit in these bytes can be used to change the behavior of the protocol. An email from Bram suggests that trailing bits should be used first, so that leading bits may be used to change the meaning of trailing bits.
info_hash: 20-byte SHA1 hash of the info key in the metainfo file. This is the same info_hash that is transmitted in tracker requests.
peer_id: 20-byte string used as a unique ID for the client. This is usually the same peer_id that is transmitted in tracker requests (but not always e.g. an anonymity option in Azureus).
In version 1.0 of the BitTorrent protocol, pstrlen = 19, and pstr = "BitTorrent protocol".

The initiator of a connection is expected to transmit their handshake immediately. The recipient may wait for the initiator's handshake, if it is capable of serving multiple torrents simultaneously (torrents are uniquely identified by their infohash). However, the recipient must respond as soon as it sees the info_hash part of the handshake (the peer id will presumably be sent after the recipient sends its own handshake). The tracker's NAT-checking feature does not send the peer_id field of the handshake.

If a client receives a handshake with an info_hash that it is not currently serving, then the client must drop the connection.

If the initiator of the connection receives a handshake in which the peer_id does not match the expected peerid, then the initiator is expected to drop the connection. Note that the initiator presumably received the peer information from the tracker, which includes the peer_id that was registered by the peer. The peer_id from the tracker and in the handshake are expected to match.


https://wiki.theory.org/BitTorrentSpecification#Handshake
         *
         */

        Ok(())
    }
}
