use std::path::{Path, PathBuf};
use crate::errors::Error;
use crate::forms::image_field_form::ImageFieldForm;
use crate::images::image_storage::ImageStorage;
use crate::util::decode_base64;

fn normalize_current_path(path: Option<&str>) -> Option<String> {
	let value = path.unwrap_or("").trim();
	if value.is_empty() {
		None
	} else {
		Some(value.to_string())
	}
}

fn file_name_from_form_path(path: &Path) -> Result<&std::ffi::OsStr, Error> {
	path.file_name().ok_or(Error::Validation)
}

pub async fn save_image_field(
	current_path: Option<&str>,
	new_value: Option<&ImageFieldForm>,
	base_dir: &Path,
) -> Result<Option<String>, Error> {
	let current = normalize_current_path(current_path);
	let Some(form) = new_value else {
		return Ok(current);
	};

	let storage = ImageStorage::new();

	if form.del.unwrap_or(false) {
		if let Some(path) = current.clone() {
			storage.delete(&PathBuf::from(path)).await?;
		}
		return Ok(None);
	}

	let Some(data_base64) = form.data_base64.as_ref() else {
		return Ok(current);
	};

	let file_name = file_name_from_form_path(&form.path)?;
	let rel_path = PathBuf::from(base_dir).join(file_name);
	let bytes = decode_base64(data_base64)?;

	if let Some(path) = current.clone() {
		storage.delete(&PathBuf::from(path)).await?;
	}

	let saved = storage.save(bytes, &rel_path).await?;
	Ok(Some(saved.display().to_string()))
}
