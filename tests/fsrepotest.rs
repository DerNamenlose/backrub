#[cfg(test)]
mod fsrepotest {
    use assert2;
    use assert_fs::prelude::*;
    use backrub::create::make_backup;
    use backrub::crypto::InputKey;
    use backrub::errors::Result;
    use backrub::fsrepository::FsRepository;
    use backrub::repository::Repository;
    use backrub::restore::restore_backup;
    use rand::prelude::*;
    use rand_distr::Exp;
    use std::fs;
    use std::path::Path;

    #[test]
    fn initialize_creates_repo_structure() -> Result<()> {
        let temp = assert_fs::TempDir::new().unwrap();
        let test_path = temp.path().to_str().unwrap();
        let repo: FsRepository = Repository::new(test_path);
        repo.initialize(InputKey::from(b"MyTestKey" as &[u8]))?;

        assert2::assert!(Path::is_file(temp.child("backrub").path()));
        assert2::assert!(Path::is_dir(temp.child("blocks").path()));
        assert2::assert!(Path::is_dir(temp.child("instances").path()));
        assert2::assert!(Path::is_dir(temp.child("keys").path()));
        assert2::assert!(
            fs::read_dir(temp.child("keys").path())
                .unwrap()
                .collect::<Vec<std::io::Result<fs::DirEntry>>>()
                .len()
                == 1
        );

        Ok(())
    }

    #[test]
    fn block_is_stored_in_repository() -> Result<()> {
        let temp = assert_fs::TempDir::new().unwrap();
        let test_path = temp.path().to_str().unwrap();
        let repo: FsRepository = Repository::new(test_path);
        repo.initialize(InputKey::from(b"MyTestKey" as &[u8]))?;
        let string = "This is a test";
        repo.add_block(string.as_bytes()).unwrap();

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

    // #[test]
    // fn object_is_represented_by_correct_block() -> Result<()> {
    //     // let temp = assert_fs::TempDir::new().unwrap();
    //     // let test_path = temp.path().to_str().unwrap();
    //     // let repo: FsRepository = Repository::new(test_path);
    //     // repo.initialize(InputKey::from(b"MyTestKey" as &[u8]))?;
    //     // let object =
    //     // let string = "This is a test";
    //     // repo.add_block(string.as_bytes()).unwrap();

    //     // assert2::let_assert!(Ok(object_id) = repo.finish());

    //     // assert2::let_assert!(
    //     //     Ok(content) = fs::read(
    //     //         temp.child("blocks")
    //     //             .child(&object_id[..2])
    //     //             .child(&object_id[2..])
    //     //             .path(),
    //     //     )
    //     // );

    //     // let mut deserializer = Deserializer::new(Cursor::new(content));
    //     // let deserialize_result: std::result::Result<BackupObject, Error> =
    //     //     Deserialize::deserialize(&mut deserializer);
    //     // assert2::let_assert!(Ok(structure) = deserialize_result);

    //     // assert2::assert!(
    //     //     structure
    //     //         == BackupObject {
    //     //             blocks: vec![String::from(
    //     //                 "3c3b66edcfe51f5b15bf372f61e25710ffc1ad3c0e3c60d832b42053a96772cf"
    //     //             )]
    //     //         }
    //     // );

    //     // Ok(())
    //     todo!()
    // }

    // #[test]
    // fn object_roundtrip_is_successful() -> Result<()> {
    //     // let mut data = vec![0; 65536];
    //     // let temp = assert_fs::TempDir::new().unwrap();
    //     // let test_path = temp.path().to_str().unwrap();
    //     // let object_id: String;
    //     // {
    //     //     let repo: FsRepository = Repository::new(test_path);
    //     //     repo.initialize(InputKey::from(b"MyTestKey" as &[u8]))?;
    //     //     let mut object = BackupObject { blocks: vec![] };
    //     //     //let mut object = repo.start_object("test").unwrap();
    //     //     let mut rnd = rand::thread_rng();
    //     //     rnd.fill_bytes(&mut data);
    //     //     object.blocks.push(repo.add_block(&data[..4096]).unwrap());
    //     //     object
    //     //         .blocks
    //     //         .push(repo.add_block(&data[4096..8192]).unwrap());
    //     //     object
    //     //         .blocks
    //     //         .push(repo.add_block(&data[8192..16384]).unwrap());
    //     //     object
    //     //         .blocks
    //     //         .push(repo.add_block(&data[16384..65536]).unwrap());
    //     //     object_id = repo.finish_object(object).unwrap();
    //     // };
    //     // // close everything and re-initialize it
    //     // let mut repo: FsRepository = Repository::new(test_path);
    //     // repo.open(InputKey::from(b"MyTestKey" as &[u8]))?;
    //     // let object = repo.open_object(&object_id).unwrap();
    //     // let object_reader = repo.open_object_reader(object)?;
    //     // let return_data: Vec<u8> = object_reader.blocks().flatten().collect();
    //     // assert2::assert!(return_data == data);

    //     // Ok(())
    //     unimplemented!()
    // }

    #[test]
    fn instance_roundtrip_is_successful() -> Result<()> {
        let source_dir = assert_fs::TempDir::new().unwrap();
        let test_path = source_dir.path().to_str().unwrap();
        print!("Initializing source directory... ");
        setup_source_dir(test_path);
        println!("Done.");
        println!("Starting backup process...");
        let repo_t = assert_fs::TempDir::new().unwrap();
        let r = repo_t.into_persistent();
        let repo_path = r.path().to_str().unwrap();

        let repo: FsRepository = Repository::new(repo_path);
        repo.initialize(InputKey::from(b"MyTestKey" as &[u8]))?;
        std::env::set_var("BACKRUB_KEY", "MyTestKey");
        make_backup(repo_path, test_path, "ThisRandomBackup")?;

        let restore_dir = assert_fs::TempDir::new().unwrap();
        let restore_path = restore_dir.path().to_str().unwrap();
        println!("Restoring backup...");
        restore_backup(repo_path, restore_path, "ThisRandomBackup")?;

        println!("Comparing source and restored path...");
        let mut all_source_files: Vec<String> = walkdir::WalkDir::new(&test_path)
            .into_iter()
            .filter_map(|e| e.ok())
            .filter(|p| p.path().is_file())
            .map(|e| String::from(e.path().strip_prefix(&test_path).unwrap().to_str().unwrap()))
            .collect();
        let mut all_restored_files: Vec<String> = walkdir::WalkDir::new(&restore_path)
            .into_iter()
            .filter_map(|e| e.ok())
            .filter(|p| p.path().is_file())
            .map(|e| {
                String::from(
                    e.path()
                        .strip_prefix(&restore_path)
                        .unwrap()
                        .to_str()
                        .unwrap(),
                )
            })
            .collect();

        assert2::assert!(all_source_files.len() != 0);
        assert2::assert!(all_restored_files.len() != 0);
        all_source_files.sort();
        all_restored_files.sort();
        assert2::assert!(all_source_files.eq(&all_restored_files));

        assert2::assert!(all_source_files.iter().zip(all_restored_files).all(
            |(source, restored)| {
                let s = source_dir.child(source);
                let sf = s.path().to_str().unwrap();
                let r = restore_dir.child(restored);
                let rf = r.path().to_str().unwrap();
                let source_file = fs::read(&sf);
                let restored_file = fs::read(&rf);
                if source_file.is_err() || restored_file.is_err() {
                    println!("Could not read one of the files: {} {}", &sf, &rf);
                    false
                } else {
                    let is_same = source_file.unwrap() == restored_file.unwrap();
                    if !is_same {
                        println!("File contents differ: {} - {}", &sf, &rf);
                        false
                    } else {
                        true
                    }
                }
            }
        ));

        Ok(())
    }

    #[test]
    fn stored_keys_are_loaded_by_the_repo() -> Result<()> {
        let repo_dir = assert_fs::TempDir::new().unwrap();
        let repo_path = repo_dir.path().to_str().unwrap();
        {
            let repo: FsRepository = Repository::new(repo_path);
            repo.initialize(InputKey::from(b"ThisIsATest" as &[u8]))?;
        }

        let mut repo: FsRepository = Repository::new(repo_path);
        repo.open(InputKey::from(b"ThisIsATest" as &[u8]))?;

        assert2::assert!(repo.keys()?.len() == 1);

        Ok(())
    }

    fn setup_source_dir(path: &str) {
        let mut rnd = rand::thread_rng();
        let filenames: Vec<String> = (0..100)
            .map(|_| format!("file-{}", rnd.next_u32()))
            .collect();
        let pathnames: Vec<String> = (0..20).map(|_| format!("dir-{}", rnd.next_u32())).collect();
        for dir in &pathnames {
            let p: std::path::PathBuf = [path, &dir].iter().collect();
            fs::create_dir_all(p).unwrap();
        }
        let exp = Exp::new(3.0).unwrap();
        for file in filenames {
            let parent_dir = pathnames.choose(&mut rnd).unwrap();
            let data_size = (rnd.sample(exp) * 1024.0 * 1024.0) as usize; // generate an exponentially distributed filesize
            let mut data = vec![0; data_size];
            rnd.fill_bytes(&mut data);
            let p: std::path::PathBuf = [path, &parent_dir, &file].iter().collect();
            fs::write(p, data).unwrap();
        }
        let zero_size_file: std::path::PathBuf = [path, "zero-file"].iter().collect();
        fs::File::create(zero_size_file).unwrap();
    }
}
