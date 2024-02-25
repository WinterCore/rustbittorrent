use url::Url;
use rand;

use crate::net::udp::send_udp_data_and_receive_response;

#[derive(Debug)]
pub struct Tracker<'a> {
    url: &'a Url
}

impl<'a> Tracker<'a> {
    pub fn new(url: &'a Url) -> Self {
        Self { url }
    }

    pub async fn connect(&self) -> Result<(), String> {
        // Build connect message
        let mut msg = [0u8; 16];
        
        // connection_id: magic number
        msg[0..8].copy_from_slice(&0x41727101980i64.to_be_bytes());

        // action: 0 for connect
        msg[8..12].copy_from_slice(&0i32.to_be_bytes());

        // transaction_id: Random 32 bit integer
        msg[12..16].copy_from_slice(&rand::random::<i32>().to_be_bytes());


        // let resp = 

        let resp = send_udp_data_and_receive_response(
            &self.url,
            &msg,
        ).await
        .map_err(|_| "Failed to connect to tracker")?;

        println!("{:?}", resp);

        Ok(())
    }
}


