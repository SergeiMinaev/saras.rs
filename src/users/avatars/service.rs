use std::path::PathBuf;
use serde::{ Serialize,Deserialize };
use log::debug;
use crate::users::avatars::db::AvatarDb;
use crate::errors::Error;
use crate::forms::image_field_form::ImageFieldForm;
use crate::images::image_storage::ImageStorage;
use crate::util::decode_base64;
use crate::db::get_pool;


#[derive(Serialize, Deserialize, Debug)]
pub struct Avatar {
	pub path: String,
}

/// Update user's avatar. To delete avatar send empty string.
pub async fn update_avatar(user_id: i32, form: &ImageFieldForm) -> Result<(), Error> {
	println!("update_avatar {user_id}");
	let _ = delete_avatar(user_id).await;
	let img_storage = ImageStorage::new();
	let bytes = decode_base64(&form.data_base64)?;
	let path = PathBuf::from("users/avatars").join(form.path.clone());
	debug!("ava form path: {}", &form.path.display());
	let path = img_storage.save(bytes, &path).await?;
	debug!("ava cloud path: {}", path.display());
	let pool = get_pool();
	let avatardb = AvatarDb::new(pool);
	avatardb.save(user_id, path.as_path().to_str().unwrap()).await?;
	Ok(())
}

pub async fn delete_avatar(user_id: i32) -> Result<(), Error> {
	let pool = get_pool();
	let avatardb = AvatarDb::new(pool);
	match avatardb.by_user_id(user_id).await {
		Err(()) => Error::Database,
		Ok(existing_avatar) => {
			let img_storage = ImageStorage::new();
			match img_storage.delete(&PathBuf::from(existing_avatar.path)).await {
				Ok(_) => {
					avatardb.delete(user_id).await;
					return Ok(())
				},
				Err(e) => return Err(e)
			}
		}
	};
	Ok(())
}
