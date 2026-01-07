use std::path::PathBuf;
use std::fs::File;
use std::io::prelude::*;
use brotli::CompressorWriter;

use uuid::Uuid;
use crate::storage::storage::Storage;
use crate::errors::Error;


pub async fn open_local_file(path: &PathBuf) -> Vec<u8> {
	let mut file = File::open(path).unwrap();
	let mut buffer: Vec<u8> = vec![];
	file.read_to_end(&mut buffer).unwrap();
	buffer
}

pub fn compress_br(data: &[u8]) -> Vec<u8> {
	let mut compressed: Vec<u8> = Vec::new();
	{
		let mut writer = CompressorWriter::new(&mut compressed, 4096, 5, 22);
		writer.write_all(data).unwrap();
	}
	compressed
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

/// Save raw `data` under `dir/UUID.ext` applying Brotli compression
/// and the correct `Content-Encoding: br` header.
///
/// Returns the final unique logical path (without the `.br` suffix)
/// inside the storage container.
pub async fn save_uuid_named_br(
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
	// `save_br` takes care of compression and path uniqueness internally.
	storage.save_br(data, &candidate).await
}

/// Save stream data into the cloud storage under `dir/UUID.ext`.
/// Uses streaming upload without brotli compression.
pub async fn save_uuid_named_stream<R>(
	storage: &Storage,
	reader: R,
	size_bytes: u64,
	dir: &str,
	ext: &str,
) -> Result<PathBuf, Error>
where
	R: Read + Send + Sync + 'static,
{
	let file_name = if ext.is_empty() {
		Uuid::new_v4().to_string()
	} else {
		format!("{}.{}", Uuid::new_v4(), ext)
	};
	let candidate = PathBuf::from(dir).join(file_name);
	let unique = storage.get_unique_path(&candidate).await?;
	storage.save_stream_fixed(reader, size_bytes, &unique).await?;
	Ok(unique)
}
