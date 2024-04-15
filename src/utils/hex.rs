use std::{fmt::Write, num::ParseIntError};

pub fn encode_hex(bytes: &[u8]) -> String {
    let mut hex_string = String::with_capacity(bytes.len() * 2);

    for byte in bytes {
        write!(&mut hex_string, "{:02x}", byte).unwrap();
    }

    hex_string
}

pub fn decode_hex(hex_string: &str) -> Result<Vec<u8>, ParseIntError> {
    (0..hex_string.len())
        .step_by(2)
        .map(|i| u8::from_str_radix(&hex_string[i..(i + 2)], 16))
        .collect()
}
