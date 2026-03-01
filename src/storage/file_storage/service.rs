use std::path::PathBuf;
use base64::engine::general_purpose;
use base64::Engine;
use crate::errors::Error;
use crate::storage::file_storage::forms::FileStorageUploadForm;
use crate::storage::storage::Storage;
use crate::util::norm_path;

fn scoped_base_path(raw: &str) -> String {
	let norm = norm_path(raw.to_string());
	let rel = norm.trim_start_matches('/');
	let scoped_rel = if rel.is_empty() {
		"files".to_string()
	} else if rel == "files" || rel.starts_with("files/") {
		rel.to_string()
	} else {
		format!("files/{rel}")
	};
	format!("/{scoped_rel}")
}

pub fn scoped_storage_path(raw: &str) -> PathBuf {
	let scoped = scoped_base_path(raw);
	PathBuf::from(scoped.trim_start_matches('/'))
}

pub async fn upload_file(form: FileStorageUploadForm) -> Result<PathBuf, Error> {
	let relative_path = match form.relative_path.as_ref() {
		"" => &form.name,
		_ => &form.relative_path,
	};
	let base_path = scoped_base_path(&form.path);
	let _path = norm_path(format!("{}/{}", base_path, relative_path.replace(" ", "-")));
	let path = PathBuf::from(_path);
	let path = path.strip_prefix("/").unwrap_or(&path).to_path_buf();
	let mut split = form.data.split(",");
	let content = split.nth(1).unwrap_or_default();
	let bytes = general_purpose::STANDARD
		.decode(content)
		.map_err(|_| Error::Decode)?;
	let storage = Storage::new();
	storage.save(bytes, &path).await
}

pub async fn delete_file(path: &PathBuf, is_dir: bool) -> Result<(), Error> {
	let storage = Storage::new();
	if is_dir {
		let mut stack: Vec<PathBuf> = vec![path.clone()];
		while let Some(dir) = stack.pop() {
			let list = storage.ls(&dir).await;
			for item in list {
				if item.ends_with('/') {
					let nested = item.trim_end_matches('/');
					if !nested.is_empty() {
						stack.push(PathBuf::from(nested));
					}
				} else {
					storage.delete(&item).await?;
				}
			}
		}
		return Ok(());
	}
	storage.delete(path).await
}
