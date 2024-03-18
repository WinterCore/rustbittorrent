use std::net::SocketAddrV4;



#[derive(Debug)]
pub struct PeerClient<'addr> {
    pub socket_addr: &'addr SocketAddrV4,
}

impl<'addr> PeerClient<'addr> {
    fn new(socket_addr: &'addr SocketAddrV4) -> Self {
        Self { socket_addr }
    }
}
