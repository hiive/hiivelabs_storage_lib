use crate::storage::sqlite_storage_container_impl::SqliteStorageContainer;
use std::any::type_name;

#[test]
fn test_package_name_for_type_camel_case() {
    let original_type_name = type_name::<std::marker::PhantomPinned>();
    let type_name =
        SqliteStorageContainer::get_package_name_for_type::<std::marker::PhantomPinned>();
    println!("{original_type_name} -> {type_name}");
    assert_eq!(type_name, "phantom_pinned");
    assert_ne!(original_type_name, type_name);
}
