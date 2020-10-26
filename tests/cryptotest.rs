#[cfg(test)]
mod fsrepotest {
    use backrub::crypto::decode_keyed_block;
    use backrub::crypto::encode_keyed_block;
    use backrub::crypto::DataEncryptionKey;
    use backrub::crypto::KeySet;
    use rand::prelude::*;
    use std::io::Cursor;

    #[test]
    fn keyed_block_roundtrip_results_in_original() {
        let mut data = vec![0; 65535];
        let mut rnd = rand::thread_rng();
        rnd.fill_bytes(&mut data);

        let mut test_key_set = KeySet::new();
        test_key_set.insert(
            1,
            DataEncryptionKey {
                created_at: 0,
                value: Vec::from(b"0123456789ABCDEF0123456789ABCDEF" as &[u8]),
            },
        );

        let mut encoded = vec![];
        encode_keyed_block(
            &mut encoded,
            &data,
            &(1, test_key_set.get(&1).unwrap().clone()),
        )
        .unwrap();
        let decoded = decode_keyed_block(&mut Cursor::new(encoded), &test_key_set).unwrap();

        assert2::assert!(data == decoded);
    }
}
