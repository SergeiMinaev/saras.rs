use std::path::PathBuf;
use base64::engine::general_purpose;
use base64::Engine;
use crate::errors::Error;
use crate::storage::img_storage::forms::ImgStorageUploadForm;
use crate::util::norm_path;
use crate::images::image_storage::ImageStorage;


pub async fn upload_img(form: ImgStorageUploadForm) -> Result<PathBuf, Error> {
	let relative_path = match form.relative_path.as_ref() {
		"" => &form.name,
		_ => &form.relative_path,
	};
	let _path = norm_path(format!("{}/{}",
		form.path.replace(" ", "-"),
		relative_path.replace(" ", "-"),
	));
	let path = PathBuf::from(_path);
	let path = path.strip_prefix("/").unwrap_or(&path).to_path_buf();
	let mut split = form.data.split(",");
	let content = split.nth(1).unwrap_or_default();
	let bytes = general_purpose::STANDARD.decode(content).unwrap();
	let img_storage = ImageStorage::builder().high().build();
	img_storage.save(bytes, &path).await
}

pub async fn delete_img(path: &PathBuf) -> Result<(), Error> {
	let img_storage = ImageStorage::new();
	img_storage.delete(&path).await
}
