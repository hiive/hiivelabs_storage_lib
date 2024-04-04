use crate::storage::unique_id_trait::UniqueId;

pub trait StorageContainer {
    fn save_data_to_package<T: UniqueId + bitcode::Encode + for<'a> bitcode::Decode<'a>>(
        &self,
        to_store: T,
        compress: bool,
    ) -> Result<String, &str>;

    fn load_data_from_package<T: UniqueId + bitcode::Encode + for<'a> bitcode::Decode<'a>>(
        &self,
        package_unique_id: &str,
    ) -> Result<T, &str>;

    fn delete_data_from_package(&self, package_unique_id: &str) -> Result<(), &str>;
    fn get_package_contents(&self) -> Vec<&str>;
}
