use argon2::{
	password_hash::{
		PasswordHash, PasswordVerifier
	},
	Argon2
};
use serde::{Serialize,Deserialize};
use crate::models::base_model::BaseModel;
use crate::models::image_field::{ ImageField };
use crate::users::users::db::UserDb;
use crate::db::get_pool;


#[derive(Serialize, Deserialize, Debug, schemars::JsonSchema)]
#[serde(rename_all = "lowercase")]
pub struct TextField {
	#[validate(length(max = 1000))]
	text: String
}

#[derive(Serialize, Deserialize, Debug, schemars::JsonSchema)]
#[serde(rename_all = "lowercase")]
pub struct User {
	pub id: u32,
	pub label: String,
	#[schemars(title = "Email", description = "Email пользователя")]
	pub email: String,
	#[serde(skip_serializing)]
	pub hash: String,
	pub is_superuser: bool,
	pub avatar: Option<ImageField>,
	pub default_avatar: Option<ImageField>,
	pub name: Option<String>,
}

impl BaseModel for User {
	const NAME: &'static str = "Пользователь";
	const NAME_PLURAL: &'static str = "Пользователи";
}

impl User {
	pub fn is_logged_with_oauth(&self) -> bool {
		return self.hash == "".to_string()
	}
	pub fn check_password(&self, pwd: String, hash: String) -> bool {
		match PasswordHash::new(&hash) {
			Err(e) => {
				println!("Unable to create PasswordHash: {e:?}");
				return false
			},
			Ok(hash) => {
				match Argon2::default().verify_password(pwd.as_bytes(), &hash) {
					Err(_) => return false,
					Ok(()) => return true,
				}
			},
		}
	}
}
