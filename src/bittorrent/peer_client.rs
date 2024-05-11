use std::{io, net::{Ipv4Addr, SocketAddr, SocketAddrV4}, time::Duration};

use tokio::{io::{AsyncReadExt, AsyncWriteExt}, net::TcpStream, time::timeout};


#[derive(Debug)]
pub struct PeerClient<'addr> {
    pub socket_addr: &'addr SocketAddr,
    pub infohash: [u8; 20],
    pub node_id: [u8; 20],
    
    stream: TcpStream,
}

impl<'addr> PeerClient<'addr> {
    pub async fn connect(
        node_id: &[u8; 20],
        socket_addr: &'addr SocketAddr,
        infohash: &[u8; 20],
    ) -> io::Result<Self> {
        let stream = timeout(Duration::from_secs(3), TcpStream::connect(socket_addr)).await??;

        Ok(Self {
            infohash: infohash.clone(),
            node_id: node_id.clone(),
            socket_addr,
            stream,
        })
    }

    pub async fn send_interested(&mut self) -> io::Result<()> {
        self.stream.write_all(&[
            0x00, 0x00, 0x00, 0x01, // Length
            0x01,
        ]).await?;

        Ok(())
    }

    pub async fn request(&mut self) -> io::Result<()> {
        Ok(())
    }

    pub async fn send_handshake(&mut self) -> io::Result<()> {
        let mut handshake_data: Vec<u8> = vec![];

        handshake_data.push(19);
        handshake_data.extend_from_slice("BitTorrent protocol".as_bytes());
        handshake_data.extend_from_slice(&[0u8; 8]);
        handshake_data.extend_from_slice(&self.infohash);
        handshake_data.extend_from_slice(&self.node_id);

        self.stream.write_all(&handshake_data).await?;
        println!("HANDSHAKE DATA: {:?}", handshake_data);

        let mut buf = [0u8; 68];
        // let mut buf = Vec::new();
        let resp = self.stream.read_exact(&mut buf).await?;

        println!("Received TCP resp: {:?} {:?}", buf, resp);

        Ok(())
    }
}
