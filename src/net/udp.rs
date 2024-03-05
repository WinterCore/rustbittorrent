use tokio::net::UdpSocket;
use url::Url;
use std::{io, net::SocketAddr};


pub async fn send_udp_packet(
    url: &Url,
    data: &[u8],
) -> io::Result<Vec<u8>> {
    // Specifying a port of 0 will make the os pick a random port for us
    let socket = UdpSocket::bind("0.0.0.0:0").await?;
    
    socket.connect(&*url.socket_addrs(|| None)?).await?;
    // let addr = "34.229.89.117:6881".parse::<SocketAddr>();
    // socket.connect(addr).await?;
    socket.send(data).await?;

    println!("Data sent: {:x?}", data);
    println!("Waiting for response");
    let mut resp_buf = [0u8; 65_535];
    let bytes_count = socket.recv(&mut resp_buf).await?;

    Ok(resp_buf[0..bytes_count].to_vec())
}
