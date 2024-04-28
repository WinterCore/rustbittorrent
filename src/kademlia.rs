
pub fn get_distance(a: &[u8; 20], b: &[u8; 20]) -> [u8; 20] {
    let mut result: [u8; 20] = [0; 20];

    for i in 0..20 {
        result[i] = a[i] ^ b[i];
    }

    result
}

pub fn distance_to_integer(distance: &[u8; 20]) -> u128 {
    u128::from_be_bytes(distance[4..].try_into().unwrap())    
}

#[cfg(test)]
mod tests {
    use crate::kademlia::get_distance;

    #[test]
    fn calculates_distance1() {
        let node1: [u8; 20] = [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1, 0];
        let node2: [u8; 20] = [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1];

        let mut result: [u8; 20] = [0; 20];

        result[18] = 1;
        result[19] = 1;

        assert_eq!(get_distance(&node1, &node2), result);
    }
}
