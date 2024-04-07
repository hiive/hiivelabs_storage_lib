#![allow(dead_code)]

pub mod storage;
pub mod string_utils;

pub mod prelude {
    pub use crate::storage::sqlite_storage_container_impl::SqliteStorageContainer;
    pub use crate::storage::storage_container_trait::StorageContainer;
    pub use crate::storage::unique_id_trait::UniqueId;
    pub use crate::string_utils::{convert_str_to_title_case, convert_str_to_underscore_case};
}
