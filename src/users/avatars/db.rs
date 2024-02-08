use lpsql::QueryParam as qp;
use crate::users::avatars::models::Avatar;
use crate::users::users::forms::UserForm;
use crate::errors::Error;
use argon2::{
	password_hash::{
		rand_core::OsRng,
		PasswordHash, PasswordHasher, PasswordVerifier, SaltString
	},
	Argon2
};

pub struct AvatarDb {}

impl AvatarDb {
	pub fn by_user_id(user_id: i32) -> Result<Avatar, ()> {
		let prms: Vec<qp> = vec![
			qp::Number(user_id)
		];
		let query = "select avatar from users_users where id = $1::INT";
		match lpsql::get_one(query, prms) {
			None => Err(()),
			Some(path) => {
				return Ok(Avatar { path: path })
			}
		}
	}
	pub fn save(user_id: i32, rel_path: &str) -> Result<(), Error> {
		let prms: Vec<qp> = vec![
			qp::Number(user_id),
			qp::String(rel_path.to_string()),
		];
		let query = "update users_users set avatar = $2::TEXT where id = $1::INT";
		lpsql::_exec(query, prms).map_err(|_| Error::Database)?;
		Ok(())
	}
	pub fn delete(user_id: i32) -> bool {
		let prms: Vec<qp> = vec![
			qp::Number(user_id)
		];
		let query = "update users_users set avatar = null where id = $1::INT";
		lpsql::_exec(query, prms).unwrap();
		true
	}
}
