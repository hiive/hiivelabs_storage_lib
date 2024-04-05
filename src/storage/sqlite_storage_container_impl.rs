use crate::storage::storage_container_trait::StorageContainer;
use crate::storage::unique_id_trait::UniqueId;
use miniz_oxide::deflate::compress_to_vec;
use rusqlite::{named_params, CachedStatement, Connection};
use std::any::type_name;
use std::fs;
use std::path::Path;
use std::str::FromStr;

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
        let table_name = SqliteStorageContainer::get_package_name_for_type::<T>();
        let package_unique_id = to_store.get_unique_id(self.mangle);
        log::info!(
            "Writing [{}] to [{}.{}]",
            package_unique_id,
            self.db_file_path,
            table_name
        );

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
                } else {
                    // compression made it smaller
                    compressed
                }
            }
            encoded
        };

        // ensure the table exists
        let conn = self.get_db_connection()?;
        let _ = self.ensure_table_exists(&conn, &table_name)?;

        // save the data to the table
        let result = conn.execute(
            &format!(
                r#"INSERT INTO {table_name} (unique_id, serialized_data, compressed)
        VALUES(:package_unique_id, :serialized_data, :compressed) ON CONFLICT (unique_id)
        DO UPDATE SET serialized_data = :serialized_data, compressed = :compressed"#
            ),
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
        Ok(package_unique_id)
    }

    fn load_data_from_package<T: UniqueId + bitcode::Encode + for<'a> bitcode::Decode<'a>>(
        &self,
        package_unique_id: &str,
    ) -> Result<T, &str> {
        let table_name = SqliteStorageContainer::get_package_name_for_type::<T>();
        log::info!(
            "Reading [{}] from [{}.{}]",
            package_unique_id,
            self.db_file_path,
            table_name
        );

        let conn = self.get_db_connection()?;
        let _ = self.ensure_table_exists(&conn, &table_name)?;

        let result = conn.query_row(
            &format!(
                r#"SELECT serialized_data, compressed FROM {table_name}
                       WHERE unique_id = :package_unique_id;"#
            ),
            named_params! {
                 ":package_unique_id": package_unique_id
            },
            |row| {
                let blob: Vec<u8> = row.get(0)?;
                let compressed: bool = row.get(1)?;
                Ok((blob, compressed))
            },
        );
        match result {
            Ok((raw_blob, compressed)) => {
                let processed_blob = {
                    if compressed {
                        miniz_oxide::inflate::decompress_to_vec(raw_blob.as_slice())
                    } else {
                        Ok(raw_blob)
                    }
                }
                .expect("Unable to process blob");

                let data_to_deserialize = processed_blob.as_slice();
                let deserialized_result = bitcode::decode::<T>(data_to_deserialize)
                    .expect("Unable to deserialize processed blob");
                Ok(deserialized_result)
            }
            Err(_) => Err("Failed to read data."),
        }
    }

    fn delete_data_from_package<T: UniqueId + bitcode::Encode + for<'a> bitcode::Decode<'a>>(
        &self,
        package_unique_id: &str,
    ) -> Result<(), &str> {
        let table_name = SqliteStorageContainer::get_package_name_for_type::<T>();
        log::info!(
            "Deleting [{}] from [{}.{}]",
            package_unique_id,
            self.db_file_path,
            table_name
        );
        let conn = self.get_db_connection()?;
        let _ = self.ensure_table_exists(&conn, &table_name)?;

        // save the data to the table
        let result = conn.execute(
            &format!(
                r#"DELETE FROM {table_name}
                       WHERE unique_id = :package_unique_id;"#
            ),
            named_params! {
                 ":package_unique_id": package_unique_id
            },
        );

        match result {
            Ok(_) => {}
            Err(_) => {
                return Err("Error deleting data from package");
            }
        }
        Ok(())
    }

    fn get_package_contents<T: UniqueId + bitcode::Encode + for<'a> bitcode::Decode<'a>>(
        &self,
    ) -> Result<Vec<String>, &str> {
        let table_name = SqliteStorageContainer::get_package_name_for_type::<T>();
        log::info!("Listing from [{}.{}]", self.db_file_path, table_name);
        let conn = self.get_db_connection()?;
        let _ = self.ensure_table_exists(&conn, &table_name)?;

        let stmt_result = conn.prepare_cached(&format!("SELECT unique_id FROM {table_name};",));
        match stmt_result {
            Ok(mut stmt) => self.get_all_field_values(&mut stmt),
            Err(_) => Err("Could not prepare statement"),
        }
    }

    fn get_packages(&self) -> Result<Vec<String>, &str> {
        log::info!("Listing packages from [{}]", self.db_file_path);

        let conn = self.get_db_connection()?;
        let stmt_result = conn.prepare_cached(
                "select tbl_name from sqlite_master where type = 'table' and name like '\\_%' escape '\\';"
            );

        match stmt_result {
            Ok(mut stmt) => self.get_all_field_values(&mut stmt),
            Err(_) => Err("Could not prepare statement"),
        }
    }
}

impl SqliteStorageContainer {
    pub fn new(db_file_path: &str, mangle: bool) -> Result<Self, &str> {
        let db_path = Path::new(db_file_path);
        if let Some(db_directory) = db_path.parent() {
            if fs::create_dir_all(db_directory).is_ok() {
                let db_file_path = String::from_str(db_file_path).unwrap();
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

    fn get_db_connection(&self) -> Result<Connection, &str> {
        let db_path = &self.db_file_path;
        let conn_result = Connection::open(Path::new(db_path));
        match conn_result {
            Ok(conn) => Ok(conn),
            Err(_) => Err("Cannot open sqlite db"),
        }
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
        let result = conn.execute(&cmd, ());
        if result.is_ok() {
            Ok(())
        } else {
            Err("Table creation error.")
        }
    }

    fn exec_vacuum(&self, conn: &Connection) {
        let _ = conn.execute("VACUUM;", ());
    }

    pub(crate) fn get_package_name_for_type<T>() -> String {
        let t_name = type_name::<T>();
        let t_name = t_name.split("::").last().unwrap_or(t_name).to_string();
        crate::string_utils::convert_str_to_underscore_case(&t_name)
    }

    fn get_all_field_values(&self, stmt: &mut CachedStatement) -> Result<Vec<String>, &str> {
        let query_results = stmt.query_map((), |row| row.get::<usize, String>(0));

        match query_results {
            Ok(query) => {
                let field_values = query.map(|u| u.unwrap()).collect();
                Ok(field_values)
            }
            Err(_) => Err("Cannot read rows"),
        }
    }
}
