use argon2::{
	password_hash::{
		rand_core::OsRng,
		PasswordHash, PasswordHasher, PasswordVerifier, SaltString
	},
	Argon2
};
use std::path::PathBuf;
use serde::{Serialize,Deserialize};
use lpsql::QueryParam as qp;
use crate::auth::sessions::Session;
use crate::users::users::input::UserInput;
use crate::models::BaseModel;
use crate::models::{ ImageField, ImageStorage };



const AVATAR_UPLOAD_TO: &'static str = "users/avatars";


#[derive(Serialize, Deserialize, Debug, schemars::JsonSchema)]
#[serde(rename_all = "lowercase")]
pub struct User {
	pub id: u32,
	#[schemars(title = "Email", description = "Email пользователя")]
	pub email: String,
	#[serde(skip_serializing)]
	pub hash: String,
	pub is_superuser: bool,
	pub avatar: Option<ImageField>,
}

impl BaseModel for User {
	const NAME: &'static str = "Пользователь";
	const NAME_PLURAL: &'static str = "Пользователи";
}

impl User {
	pub fn is_logged_with_oauth(&self) -> bool {
		return self.hash == "".to_string()
	}
	pub fn create(data: UserInput) -> Option<i32> {
		if validator::validate_email(&data.email) == false { return None::<i32> }
		let argon2 = Argon2::default();
		let salt = SaltString::generate(&mut OsRng);
		let pwd = data.pwd.unwrap();
		let hash = argon2.hash_password(&pwd.into_bytes(), &salt).unwrap().to_string();
		let prms: Vec<qp> = vec![
			qp::String(data.email.to_string()),
			qp::String(hash.to_string()),
		];
		let query = "insert into users_users (email, hash) values ($1::TEXT, $2::TEXT) returning id";
		match lpsql::get_one(query, prms) {
			None => return None::<i32>,
			Some(id) => {
				return Some(id.parse().unwrap())
			}
		};

	}
	pub async fn update(id: i32, data: UserInput) -> bool {
		//println!("input {data:?}");
		let prms: Vec<qp> = vec![
			qp::Number(id),
			qp::String(data.email),
			qp::Bool(data.is_superuser),
		];
		let q = "update users_users set email = $2::TEXT, is_superuser = $3::BOOL where id = $1::INT";
		if lpsql::exec(q, prms) == false { return false }
		User::update_avatar(id, data.avatar).await
	}
	pub async fn update_avatar(id: i32, avatar: Option<String>) -> bool {
		match avatar {
			None => return true,
			Some(avatar) => {
				// Empty string means deletion.
				let prms: Vec<qp> = vec![qp::Number(id)];
				let q = "select avatar from users_users where id = $1::INT";
				let existing_path: String = lpsql::get_one(q, prms).unwrap();
				println!("existing path? {existing_path}");
				if existing_path != "" {
					ImageStorage::delete(&existing_path).await;
				}
				if avatar == "" {
					if existing_path != "" {
						let prms: Vec<qp> = vec![qp::Number(id)];
						let q = "update users_users set avatar = null where id = $1::INT";
						return lpsql::exec(q, prms)
					} else {
						return true
					}
				} else {
					let mut path: PathBuf = ImageStorage::save_from_base64(
							&avatar, AVATAR_UPLOAD_TO).await;
					let stem = format!("{}", path.file_stem().unwrap().to_str().unwrap());
					path.set_file_name(stem);
					let prms: Vec<qp> = vec![
						qp::Number(id),
						qp::String(path.to_string_lossy().to_string())
					];
					let q = "update users_users set avatar = $2::TEXT where id = $1::INT";
					return lpsql::exec(q, prms)
				}
			}
		}
	}
	pub fn delete(id: i32) -> bool {
	   let prms: Vec<qp> = vec![qp::Number(id)];
	   let q = "delete from users_users where id = $1::INT";
	   lpsql::exec(q, prms)
	}
	pub fn by_id(id: i32) -> Option<User> {
		let prms: Vec<qp> = vec![
			qp::Number(id)
		];
		let query = "select row_to_json(data) from (
			select id, email as label, email, hash, is_superuser,
			case when users.avatar is not null then
				json_build_object('path', users.avatar)
			else null end as avatar
			from users_users as users where id = $1::INT
		) data";
		match lpsql::get_one(query, prms) {
			None => None::<User>,
			Some(v) => {
				return serde_json::from_str(&v).unwrap();
			}
		};
		None
	}
	pub fn by_email(email: String) -> Option<User> {
		let prms: Vec<qp> = vec![
			qp::String(email)
		];
		let query = "select row_to_json(data) from (\
			select id, email, hash, is_superuser from users_users where email = $1::TEXT \
		) data";
		match lpsql::get_one(query, prms) {
			None => None::<User>,
			Some(v) => {
				return serde_json::from_str(&v).unwrap();
			}
		};
		None
	}
	pub fn by_session_id(sess_id: &String) -> Option<User> {
		let prms: Vec<qp> = vec![
			qp::String(sess_id.to_string())
		];
		let query = "select row_to_json(data) from (\
			select usr.id, email, hash, is_superuser \
			from users_users as usr join auth_sessions as session \
			on usr.id = session.user_id where session.id = $1::BYTEA \
			and session.expires > now()
		) data";
		match lpsql::get_one(query, prms) {
			None => None::<User>,
			Some(v) => {
				return serde_json::from_str(&v).unwrap();
			}
		};
		None
	}
	pub fn check_password(&self, pwd: String) -> bool {
		match PasswordHash::new(&self.hash) {
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
	pub fn add_session(&self) -> Option<Session> {
		let prms: Vec<qp> = vec![
			qp::String(self.id.to_string())
		];
		let query = "insert into auth_sessions (user_id) values ($1::INT) returning id";
		match lpsql::get_one(query, prms) {
			None => None::<Session>,
			Some(id) => {
				return Session::by_id(id)
			}
		}
	}
	pub fn all() -> Vec<User> {
		let mut r: Vec<User> = vec![];
		let prms: Vec<qp> = vec![];
		let query = "select row_to_json(data) from (\
			select id, email, hash, is_superuser from users\
		) data";
		match lpsql::_exec(query, prms) {
			Err(e) => println!("ERR: {e}"),
			Ok(resp) => {
				for u in resp {
					r.push(serde_json::from_str(&u).unwrap());
				}
			}
		}
		return r
	}
	pub fn create_and_get(data: UserInput) -> Option<User> {
	  match User::create(data) {
		None => return None,
		Some(id) => {
		  return User::by_id(id);
		}
	  }
	}
	pub async fn update_and_get(id: i32, data: UserInput) -> Option<User> {
	  match User::update(id, data).await {
		false => return None,
		true => return User::by_id(id),
	  }
	}


}
