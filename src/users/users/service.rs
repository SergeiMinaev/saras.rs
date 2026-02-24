use crate::users::users::models::User;
use crate::users::users::db::UserDb;
use crate::users::avatars::db::AvatarDb;
use crate::users::avatars::service::{update_avatar,delete_avatar};
use crate::users::users::forms::UserForm;
use crate::errors::Error;
use crate::db::get_pool;
use crate::validation::validate_pwd;


pub async fn create_user(user_form: UserForm) -> Result<User, Error> {
	let pool = get_pool();
	let userdb = UserDb::new(pool.clone());
	let user = userdb.create_and_get(user_form.clone()).await.ok_or(Error::Database)?;
	if let Some(ref avatar) = user_form.avatar {
		if avatar.data_base64.is_some() {
			update_avatar(user.id.try_into().unwrap(), &avatar).await?;
		}
	}
	Ok(user)
}


pub async fn update_user(id: i32, user_form: UserForm) -> Result<User, Error> {
	let pool = get_pool();
	if let Some(ref avatar) = user_form.avatar {
		if avatar.del.unwrap_or(false) {
			let avatardb = AvatarDb::new(pool.clone());
			let avatar = avatardb.by_user_id(id).await.unwrap();
			if avatar.path == "" {
				println!("no ava");
			} else {
				println!("go delete ava");
				delete_avatar(id).await?;
			}
		} else if avatar.data_base64.is_some() {
			println!("go update ava");
			update_avatar(id, &avatar).await?;
		}
	} else {
		println!("dont update ava");
	}
	let userdb = UserDb::new(pool.clone());
	userdb.update_and_get(id, user_form).await.ok_or(Error::Database)
}

pub async fn delete_user(id: i32) -> Result<(), Error> {
	let pool = get_pool();
	let userdb = UserDb::new(pool.clone());
	if userdb.delete(id).await == true {
		Ok(())
	} else {
		Err(Error::Database)
	}
}

pub async fn set_password(id: i32, pwd: &str) -> Result<(), Error> {
	if let Err(_e) = validate_pwd(pwd) {
		return Err(Error::Validation);
	}
	let pool = get_pool();
	let userdb = UserDb::new(pool.clone());
	if userdb.set_password(id, pwd).await {
		Ok(())
	} else {
		Err(Error::Database)
	}
}
