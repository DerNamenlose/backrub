#[cfg(test)]
mod fsrepotest {
    use backrub::blockcache::BlockCache;
    use backrub::repository::BackupBlockId;

    #[test]
    fn blocks_are_recognized_in_the_cache() {
        let temp = assert_fs::TempDir::new().unwrap();
        let test_path = temp.path();
        let cache = backrub::blockcache::open(test_path).unwrap();
        let bid = BackupBlockId::from_bytes(b"01234567012345670123456701234567").unwrap();
        cache.add_block(&b"abcdefg".to_vec(), &bid).unwrap();
        assert2::let_assert!(
            Ok(Some(existing_bid)) = cache.get_backup_block_id(&b"abcdefg".to_vec())
        );
        assert2::assert!(existing_bid == bid);
        assert2::let_assert!(
            Ok(Option::<BackupBlockId>::None) = cache.get_backup_block_id(&b"zyxwvu".to_vec())
        );
    }
}
