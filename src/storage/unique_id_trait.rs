pub trait UniqueId {
    fn get_unique_id(&self, mangle:bool) -> String;
}