#[cfg(test)]
mod fsrepotest {
    use backrub::blockcache::BlockCache;

    #[test]
    fn blocks_are_recognized_in_the_cache() {
        let temp = assert_fs::TempDir::new().unwrap();
        let test_path = temp.path().to_str().unwrap();
        let cache = backrub::blockcache::open(test_path).unwrap();
        cache.add_block(&b"abcdefg".to_vec()).unwrap();
        assert2::let_assert!(Ok(true) = cache.has_block(&b"abcdefg".to_vec()));
        assert2::let_assert!(Ok(false) = cache.has_block(&b"zyxwvu".to_vec()));
    }
}
