#[cfg(test)]
mod fsrepotest {
    use assert2;
    use assert_fs::prelude::*;
    use backrub::backupobject::BackupObject;
    use backrub::fsrepository::FsRepository;
    use backrub::repository::Repository;
    use rand::prelude::*;
    use rmp_serde::decode::Error;
    use rmp_serde::Deserializer;
    use serde::Deserialize;
    use std::fs;
    use std::io::Cursor;
    use std::path::Path;

    #[test]
    fn initialize_creates_repo_structure() -> std::io::Result<()> {
        let temp = assert_fs::TempDir::new().unwrap();
        let test_path = temp.path().to_str().unwrap();
        let repo: FsRepository = Repository::new(test_path);
        repo.initialize()?;

        assert2::assert!(Path::is_file(temp.child("backrub").path()));
        assert2::assert!(Path::is_dir(temp.child("blocks").path()));
        assert2::assert!(Path::is_dir(temp.child("instances").path()));

        Ok(())
    }

    #[test]
    fn block_is_stored_in_repository() -> std::io::Result<()> {
        let temp = assert_fs::TempDir::new().unwrap();
        let test_path = temp.path().to_str().unwrap();
        let repo: FsRepository = Repository::new(test_path);
        repo.initialize()?;
        let mut object = repo.start_object("test").unwrap();
        let string = "This is a test";
        object.add_block(string.as_bytes()).unwrap();

        assert2::assert!(Path::is_file(
            temp.child("blocks/3c/3b66edcfe51f5b15bf372f61e25710ffc1ad3c0e3c60d832b42053a96772cf")
                .path()
        ));
        let block_content = fs::read(
            temp.child("blocks/3c/3b66edcfe51f5b15bf372f61e25710ffc1ad3c0e3c60d832b42053a96772cf")
                .path(),
        )
        .unwrap();
        assert2::assert!(block_content == string.as_bytes());

        Ok(())
    }

    #[test]
    fn object_is_represented_by_correct_block() -> std::io::Result<()> {
        let temp = assert_fs::TempDir::new().unwrap();
        let test_path = temp.path().to_str().unwrap();
        let repo: FsRepository = Repository::new(test_path);
        repo.initialize()?;
        let mut object = repo.start_object("test").unwrap();
        let string = "This is a test";
        object.add_block(string.as_bytes()).unwrap();

        assert2::let_assert!(Ok(object_id) = object.finish());

        assert2::let_assert!(
            Ok(content) = fs::read(
                temp.child("blocks")
                    .child(&object_id[..2])
                    .child(&object_id[2..])
                    .path(),
            )
        );

        let mut deserializer = Deserializer::new(Cursor::new(content));
        let deserialize_result: Result<BackupObject, Error> =
            Deserialize::deserialize(&mut deserializer);
        assert2::let_assert!(Ok(structure) = deserialize_result);

        assert2::assert!(
            structure
                == BackupObject {
                    name: String::from("test"),
                    blocks: vec![String::from(
                        "3c3b66edcfe51f5b15bf372f61e25710ffc1ad3c0e3c60d832b42053a96772cf"
                    )]
                }
        );

        Ok(())
    }

    #[test]
    fn object_roundtrip_is_successful() -> std::io::Result<()> {
        let mut data = vec![0; 65536];
        let temp = assert_fs::TempDir::new().unwrap();
        let test_path = temp.path().to_str().unwrap();
        let object_id: String;
        {
            let repo: FsRepository = Repository::new(test_path);
            repo.initialize()?;
            let mut object = repo.start_object("test").unwrap();
            let mut rnd = rand::thread_rng();
            rnd.fill_bytes(&mut data);
            object.add_block(&data[..4096]).unwrap();
            object.add_block(&data[4096..8192]).unwrap();
            object.add_block(&data[8192..16384]).unwrap();
            object.add_block(&data[16384..65536]).unwrap();
            object_id = object.finish().unwrap();
        };
        // close everything and re-initialize it
        let repo: FsRepository = Repository::new(test_path);
        repo.initialize()?;
        let object = repo.open_object(&object_id).unwrap();
        let return_data: Vec<u8> = object.blocks().flatten().collect();
        assert2::assert!(return_data == data);

        Ok(())
    }
}
