use std::path::PathBuf;
use serde_json::{ json };
use serde::{ Serialize,Deserialize };
use validator::{ Validate };
use log::debug;
use crate::users::users::models::User;
use crate::users::users::db::UserDb;
use crate::http::{ Request };
use crate::users::users::forms::{ CreateUserForm, UpdateUserForm };
use crate::validation::ValidationErrors;
//use crate::storage::factory::{ StorageType, StorageFactory };
use crate::users::avatars::db::AvatarDb;
use crate::errors::Error;
use crate::forms::image_field_form::ImageFieldForm;
use crate::images::image_storage::ImageStorage;
use crate::storage::storage::Storage;
use crate::util::decode_base64;


#[derive(Serialize, Deserialize, Debug)]
pub struct Avatar {
	pub path: String,
}

/// Update user's avatar. To delete avatar send empty string.
pub async fn update_avatar(user_id: i32, form: &ImageFieldForm) -> Result<(), Error> {
	println!("update_avatar {user_id}");
	delete_avatar(user_id).await;
	let img_storage = ImageStorage::new();
	let bytes = decode_base64(&form.data_base64)?;
	let mut path = PathBuf::from("users/avatars").join(form.path.clone());
	debug!("ava form path: {}", &form.path.display());
	let path = img_storage.save(bytes, &path).await?;
	debug!("ava cloud path: {}", path.display());
	AvatarDb::save(user_id, path.as_path().to_str().unwrap())?;
	Ok(())
}

pub async fn delete_avatar(user_id: i32) -> Result<(), Error> {
	match AvatarDb::by_user_id(user_id) {
		Err(()) => Error::Database,
		Ok(existing_avatar) => {
			let img_storage = ImageStorage::new();
			match img_storage.delete(&PathBuf::from(existing_avatar.path)).await {
				Ok(_) => {
					AvatarDb::delete(user_id);
					return Ok(())
				},
				Err(e) => return Err(e)
			}
		}
	};
	Ok(())
}
