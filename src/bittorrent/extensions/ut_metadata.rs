use crate::utils::bencode::BencodeValue;
use crate::bittorrent::extensions::Extension;

#[derive(Debug)]
pub struct UTMetadata {
    data: Vec<u8>,
    total_size: Option<i64>,
}

impl Extension for UTMetadata {
    const NAME: &'static str = "ut_metadata";

    async fn process_packet(&self, data: &[u8]) -> () {

    }
}

impl UTMetadata {
    const MAX_PIECE_SIZE: i64 = 16_384;

    fn new() -> UTMetadata {
        Self {
            data: Vec::new(),
            total_size: None,
        }
    }

    fn next_piece_index(&self) -> i64 {
        if let Some(total_size) = self.total_size {
            let total_piece_count = (total_size as f64 / Self::MAX_PIECE_SIZE as f64) as i64;
            let downloaded_pieces = self.data.len() as i64 / Self::MAX_PIECE_SIZE;

            return total_piece_count - downloaded_pieces;
        }


        0
    }

    fn get_request_payload(&self) -> BencodeValue {
        let index = self.next_piece_index();
    }
}
