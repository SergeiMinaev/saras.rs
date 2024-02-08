//use crate::storage::interface::Storage;
//use crate::storage::engines::selectel::SelectelStorage;
//
//
//pub enum StorageType {
//	Selectel,
//}
//
//pub struct StorageFactory;
//
//impl StorageFactory {
//	pub fn get_storage(storage_type: StorageType) -> Box<dyn Storage + Send + Sync> {
//		match storage_type {
//			StorageType::Selectel => Box::new(SelectelStorage::new()),
//		}
//	}
//}
