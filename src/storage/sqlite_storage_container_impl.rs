use crate::storage::storage_container_trait::StorageContainer;
use crate::storage::unique_id_trait::UniqueId;
use miniz_oxide::deflate::compress_to_vec;
use rusqlite::{named_params, params, Connection};
use std::any::type_name;
use std::fs;
use std::path::Path;
use std::str::FromStr;
use miniz_oxide::inflate::DecompressError;

pub struct SqliteStorageContainer {
    mangle: bool,
    db_file_path: String,
}

impl StorageContainer for SqliteStorageContainer {
    fn save_data_to_package<T: UniqueId + bitcode::Encode + for<'a> bitcode::Decode<'a>>(
        &self,
        to_store: T,
        compress: bool,
    ) -> Result<String, &str> {
        let package_unique_id = to_store.get_unique_id(self.mangle);
        log::info!("Writing [{}] to [{}]", package_unique_id, self.db_file_path);

        // check compression flag and compress the data
        // if it makes it smaller.
        let mut was_compressed = compress;
        let serialized_data = {
            let mut encoded = bitcode::encode(&to_store);
            if compress {
                let compressed = compress_to_vec(encoded.as_slice(), 6);
                encoded = if encoded.len() <= compressed.len() {
                    // don't store the compressed version
                    // if it's not any smaller.
                    was_compressed = false;
                    encoded
                }
                else {
                    // compression made it smaller
                    compressed
                }
            }
            encoded
        };
        // ensure the table exists
        let table_name = SqliteStorageContainer::get_package_name_for_type::<T>();
        let conn = self.get_db_connection();
        let result = self.ensure_table_exists(&conn, table_name.as_str());
        match result {
            Ok(_) => {}
            Err(e) => {
                conn.close().expect("Error closing database");
                return Err(e);
            }
        }
        // save the data to the table
        let result = conn.execute(
            format!(
                r#"INSERT INTO {table_name} (unique_id, serialized_data, compressed)
        VALUES(:package_unique_id, :serialized_data, :compressed) ON CONFLICT (unique_id)
        DO UPDATE SET serialized_data = :serialized_data, compressed = :compressed"#
            )
            .as_str(),
            named_params! {
                ":package_unique_id": package_unique_id,
                ":serialized_data": serialized_data,
                ":compressed": was_compressed,
            },
        );
        match result {
            Ok(_) => {}
            Err(_) => {
                return Err("Error storing data to package");
            }
        }
        conn.close().expect("Error closing database");
        Ok(package_unique_id)
    }

    fn load_data_from_package<T: UniqueId + bitcode::Encode + for<'a> bitcode::Decode<'a>>(
        &self,
        package_unique_id: &str,
    ) -> Result<T, &str> {
        log::info!(
            "Reading [{}] from [{}]",
            package_unique_id,
            self.db_file_path
        );
        let table_name = SqliteStorageContainer::get_package_name_for_type::<T>();
        let conn = self.get_db_connection();

        let result = conn.query_row(
            format!(
                r#"SELECT serialized_data, compressed FROM {table_name}
                       WHERE unique_id = :package_unique_id;"#
            )
            .as_str(),
            named_params! {
                 ":package_unique_id": package_unique_id
            },
            |row| {
                let blob: Vec<u8> = row.get(0)?;
                let compressed: bool = row.get(1)?;
                Ok((blob, compressed))
            }
        );
        match result {
            Ok((raw_blob, compressed)) => {

                let processed_blob = {
                    if compressed {
                        miniz_oxide::inflate::decompress_to_vec(raw_blob.as_slice())
                    }
                    else {
                        Ok(raw_blob)
                    }
                }.expect("Unable to process blob");

                let data_to_deserialize = processed_blob.as_slice();
                let deserialized_result = bitcode::decode::<T>(&data_to_deserialize)
                    .expect("Unable to deserialize processed blob");
                Ok(deserialized_result)
                // match deserialized_result {
                //     Ok(deserialized) => {
                //         deserialized
                //     }
                //     Err(_) => {
                //         Err("Error decoding processed blob")
                //     }
                // }
                // todo!()
            }
            Err(_) => {
                Err("Failed to read data.")
            }
        }
    }

    fn delete_data_from_package(&self, package_unique_id: &str) -> Result<(), &str> {
        todo!()
    }
    fn get_package_contents(&self) -> Vec<&str> {
        todo!()
    }
}

impl SqliteStorageContainer {
    pub fn new(db_file_path: &str, mangle: bool) -> Result<Self, &str> {
        let db_path = Path::new(db_file_path);
        if let Some(db_directory) = db_path.parent() {
            if fs::create_dir_all(db_directory).is_ok() {
                let db_file_path = String::from_str(db_file_path).expect("Invalid db_file_path");
                Ok(Self {
                    db_file_path,
                    mangle,
                })
            } else {
                Err("Unable to find/create directory path.")
            }
        } else {
            Err("Invalid directory path")
        }
    }

    fn get_db_connection(&self) -> Connection {
        let db_path = self.db_file_path.as_str();
        Connection::open(Path::new(db_path)).expect("Cannot open sqlite db")
    }

    fn ensure_table_exists(&self, conn: &Connection, table_name: &str) -> Result<(), &str> {
        let cmd = format!("create table if not exists {table_name}(")
            // id primary key
            + "_id integer not null primary key autoincrement, "
            // unique id for retrieval
            + "unique_id varchar(256) not null unique, "
            // compressed flag
            + "compressed int not null, "
            // serialized data
            + "serialized_data blob "
            + ");";
        let result = conn.execute(cmd.as_str(), ());
        if result.is_ok() {
            Ok(())
        } else {
            Err("Table creation error.")
        }
    }

    fn exec_vacuum(&self, conn: &Connection) {
        conn.execute("VACUUM;", ()).expect("Vacuum command failed");
    }

    pub(crate) fn get_package_name_for_type<T>() -> String {
        let t_name = type_name::<T>();
        let t_name = t_name.split("::").last().unwrap_or(t_name).to_string();
        crate::string_utils::convert_str_to_underscore_case(t_name.as_str())
    }
}
