use crate::users::users::models::User;
use crate::users::users::db::UserDb;
use crate::users::avatars::db::AvatarDb;
use crate::validation::ValidationErrors;
use crate::users::avatars::service::{update_avatar,delete_avatar};
use crate::users::users::forms::UserForm;
use crate::errors::Error;
use crate::db::get_pool;


pub async fn create_user(user_form: UserForm) -> Result<User, ValidationErrors> {
	let pool = get_pool();
	let userdb = UserDb::new(pool.clone());
	Ok(userdb.create_and_get(user_form).await.unwrap())
}


pub async fn update_user(id: i32, user_form: UserForm) -> Result<User, Error> {
	let pool = get_pool();
	if let Some(ref avatar) = user_form.avatar {
		if avatar.path.as_os_str().is_empty() {
			let avatardb = AvatarDb::new(pool.clone());
			let avatar = avatardb.by_user_id(id).await.unwrap();
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
	let userdb = UserDb::new(pool.clone());
	Ok(userdb.update_and_get(id, user_form).await.unwrap())
}

