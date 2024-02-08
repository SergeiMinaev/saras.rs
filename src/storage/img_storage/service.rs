use std::path::PathBuf;
use serde::Deserialize;
use serde_json::{ json };
use base64::engine::general_purpose;
use base64::Engine;
use validator::{ Validate };
use crate::users::users::models::User;
use crate::users::users::db::UserDb;
use crate::users::avatars::db::AvatarDb;
use crate::http::{ Request };
use crate::users::users::forms::{ CreateUserForm, UpdateUserForm };
use crate::validation::ValidationErrors;
use crate::users::avatars::service::{update_avatar,delete_avatar};
use crate::users::users::forms::UserForm;
use crate::errors::Error;
use crate::storage::img_storage::forms::ImgStorageUploadForm;
use crate::util::norm_path;
use crate::images::image_storage::ImageStorage;


pub async fn upload_img(form: ImgStorageUploadForm) -> Result<(), Error> {
	let relativePath = match form.relativePath.as_ref() {
		"" => &form.name,
		_ => &form.relativePath,
	};
	let _path = norm_path(format!("{}/{}",
		form.path.replace(" ", "-"),
		relativePath.replace(" ", "-"),
	));
	let path = PathBuf::from(_path);
	let path = path.strip_prefix("/").unwrap_or(&path).to_path_buf();
	let mut split = form.data.split(",");
	let content = split.nth(1).unwrap_or_default();
	let bytes = general_purpose::STANDARD.decode(content).unwrap();
	let img_storage = ImageStorage::new();
	let path = img_storage.save(bytes, &path).await;
	Ok(())
}

pub async fn delete_img(path: &PathBuf) -> Result<(), Error> {
	let img_storage = ImageStorage::new();
	img_storage.delete(&path).await
}
