#[cfg(test)]
mod tests {
    use data_encoding::BASE32_NOPAD_NOCASE as BASE32;
    use iroh::PublicKey;
    use rand::{self, TryRngCore};

    #[test]
    fn test_base32() {
        for _ in 0..1000000 {
            let mut bytes = [0u8; 32];
            rand::rng().try_fill_bytes(&mut bytes).unwrap();
            let node_id = match PublicKey::from_bytes(&bytes) {
                Ok(pk) => pk,
                Err(_) => {
                    continue;
                }
            };

            let encoded = BASE32.encode(node_id.as_bytes()).to_lowercase();
            let decoded = BASE32.decode(encoded.as_bytes()).unwrap();

            // let hex_node_id: String = node_id
            //     .as_bytes()
            //     .iter()
            //     .map(|b| format!("{:02x}", b))
            //     .collect();
            // let hex_decoded: String = decoded.iter().map(|b| format!("{:02x}", b)).collect();
            // println!("node_id: {:?}", hex_node_id.to_lowercase());
            // println!("encoded: {:?}", encoded.to_lowercase());
            // println!("decoded: {:?}", hex_decoded);
            assert_eq!(node_id.as_ref(), decoded.as_slice());
        }
    }
}
