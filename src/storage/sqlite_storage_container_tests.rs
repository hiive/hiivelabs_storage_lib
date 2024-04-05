use crate::prelude::{StorageContainer, UniqueId};
use crate::storage::sqlite_storage_container_impl::SqliteStorageContainer;
use bitcode::{Decode, Encode};
use rand::prelude::StdRng;
use rand::{Rng, SeedableRng};
use sha2::{Digest, Sha256};
use std::any::type_name;
use std::collections::HashMap;
use std::time::Instant;
use uuid::Uuid;

pub(crate) fn setup_test_logger() {
    #[cfg(debug_assertions)]
    {
        let _ = env_logger::Builder::new()
            .write_style(env_logger::WriteStyle::Always)
            // Include all events in tests
            .filter_level(log::LevelFilter::max())
            // Ensure events are captured by `cargo test`
            .is_test(true)
            .format_target(false)
            // Ignore errors initializing the logger if tests race to configure it
            .try_init();

        // println!("Logger initialized.")
    }
}

#[test]
fn test_package_name_for_type_camel_case() {
    let original_type_name = type_name::<std::marker::PhantomPinned>();
    let type_name =
        SqliteStorageContainer::get_package_name_for_type::<std::marker::PhantomPinned>();
    println!("{original_type_name} -> {type_name}");
    assert_eq!(type_name, "phantom_pinned");
    assert_ne!(original_type_name, type_name);
}

#[derive(Debug, Encode, Decode, PartialEq)]
struct TestVec {
    data: Vec<u8>,
}

impl UniqueId for TestVec {
    fn get_unique_id(&self, mangle: bool) -> String {
        let bytes = self.data.get(0..16).unwrap();
        let uuid = Uuid::from_slice(bytes).unwrap().to_string();
        if mangle {
            format!("m!{uuid}")
        } else {
            format!("_!{uuid}")
        }
    }
}

impl TestVec {
    pub(crate) fn new() -> Self {
        Self {
            data: generate_random_vector(64 * 32), // vec![0; 1024]
        }
    }
    fn get_hash(&self) -> String {
        hex::encode(Sha256::digest(&self.data))
    }
}

fn generate_random_vector(length: usize) -> Vec<u8> {
    let mut rng = StdRng::from_rng(rand::thread_rng()).unwrap();
    (0..length).map(|_| rng.gen()).collect()
}

fn set_up_test_db(
    entry_count: u16,
    mangle: bool,
    compress: bool,
) -> (SqliteStorageContainer, HashMap<String, String>) {
    let db_name = "test.db";
    // ensure the db is deleted
    let result = std::fs::remove_file(db_name);

    // create the db
    let db = SqliteStorageContainer::new(db_name, mangle).unwrap();

    let mut uuid_to_hash_map = HashMap::new();
    // let's create some records
    for _ in 0..entry_count {
        let test_vec = TestVec::new();
        // let u = test_vec.get_unique_id(mangle);
        // let h = test_vec.get_hash();
        uuid_to_hash_map.insert(test_vec.get_unique_id(mangle), test_vec.get_hash());
        let _ = db.save_data_to_package(test_vec, compress);
    }

    (db, uuid_to_hash_map)
}

#[test]
fn test_read_write() {
    // setup_test_logger();

    let mangle = true;
    let compress = true;
    let (db, uuids_and_hashes) = set_up_test_db(500, mangle, compress);
    // let _ = uuids_and_hashes.drain(..).map(|(u, h)| uuid_to_hash_map.insert(u, h));

    let package_contents_result = db.get_package_contents::<TestVec>();
    match package_contents_result {
        Ok(package_contents) => {
            // check length
            assert_eq!(package_contents.len(), uuids_and_hashes.len());

            let start = Instant::now(); // Start timing
            for package_id in package_contents {
                // check package is there
                assert!(uuids_and_hashes.contains_key(&package_id));
                let package_hash = uuids_and_hashes.get(&package_id).expect("Can't find Hash");

                // println!("{package_id} -> {package_hash}");

                // load package and compare hash
                let data = db
                    .load_data_from_package::<TestVec>(&package_id)
                    .expect("Can't load data");

                let data_id = data.get_unique_id(mangle);
                let data_hash = &data.get_hash();

                assert_eq!(package_id, data_id);
                assert_eq!(package_hash, data_hash);
            }

            let duration = start.elapsed();
            // log::info!("Saved and Loaded in {duration:?}");
            println!("Saved and Loaded in {duration:?}");
        }
        Err(e) => {
            println!("{e}");
        }
    }
}
