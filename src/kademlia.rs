
pub fn get_distance(a: &[u8; 20], b: &[u8; 20]) -> Vec<u8> {
    let mut result: [u8; 20] = [0; 20];

    for i in (0..20).rev() {
        result[i] = a[i] ^ b[i];
    }

    result.to_vec()
}
