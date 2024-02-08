use serde::Deserialize;
use serde_json::{ json };
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


pub fn create_user(user_form: UserForm) -> Result<User, ValidationErrors> {
	Ok(UserDb::create_and_get(user_form).unwrap())
}


pub async fn update_user(id: i32, user_form: UserForm) -> Result<User, Error> {
	if let Some(ref avatar) = user_form.avatar {
		if avatar.path.as_os_str().is_empty() {
			let avatar = AvatarDb::by_user_id(id).unwrap();
			if avatar.path == "" {
				println!("no ava");
			} else {
				println!("go delete ava");
				delete_avatar(id).await?;
			}
		} else {
			println!("go update ava");
			update_avatar(id, &avatar).await?;
		}
	} else {
		println!("dont update ava");
	}
	Ok(UserDb::update_and_get(id, user_form).await.unwrap())
}

