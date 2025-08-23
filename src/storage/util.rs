use std::path::PathBuf;
use std::fs::File;
use std::io::prelude::*;

use uuid::Uuid;
use crate::storage::storage::Storage;
use crate::errors::Error;


pub async fn open_local_file(path: &PathBuf) -> Vec<u8> {
	let mut file = File::open(path).unwrap();
	let mut buffer: Vec<u8> = vec![];
	file.read_to_end(&mut buffer).unwrap();
	buffer
}

/// Save `data` into the cloud `storage` under `dir/UUID.ext`.
/// Returns the final unique path inside the container.
///
/// `dir` example: "orig" or "uploads".
/// `ext` example: "webp", "png"; pass an empty string for no extension.
///
/// The file name is based on `Uuid::new_v4()` and made collision-safe
/// via `Storage::get_unique_path`.
pub async fn save_uuid_named(
	storage: &Storage,
	data: Vec<u8>,
	dir: &str,
	ext: &str,
) -> Result<PathBuf, Error> {
	let file_name = if ext.is_empty() {
		Uuid::new_v4().to_string()
	} else {
		format!("{}.{}", Uuid::new_v4(), ext)
	};
	let candidate = PathBuf::from(dir).join(file_name);
	let unique = storage.get_unique_path(&candidate).await?;
	storage.save(data, &unique).await
}
