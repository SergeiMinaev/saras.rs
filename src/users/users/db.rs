use lpsql::QueryParam as qp;
use crate::lpsql::Lpsql;
use crate::users::users::models::User;
use crate::users::users::forms::UserForm;
use argon2::{
	password_hash::{
		rand_core::OsRng,
		PasswordHash, PasswordHasher, PasswordVerifier, SaltString
	},
	Argon2
};
use once_cell::sync::Lazy;
use std::sync::RwLock;
use serde::Deserialize;


pub static lpsql: Lazy<Lpsql> = Lazy::new(|| {
    Lpsql::new(None)
});


pub struct UserDb {}


impl UserDb {
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
		match lpsql.get_one(query, prms) {
			None => None::<User>,
			Some(v) => {
				serde_json::from_str(&v).unwrap()
			}
		}
	}
	pub fn total_count() -> i32 {
	  let q = "select count(*) from users_users";
	  let p: Vec<qp> = vec![];
	  lpsql.get_one(q, p).unwrap().parse().unwrap()
	}
	pub fn page(offset: i32, size: i32) -> Vec<User> {
		let mut r: Vec<User> = vec![];
		let prms: Vec<qp> = vec![
		  qp::Number(offset),
		  qp::Number(size),
		];
		let query = "select row_to_json(data) from (
			select id, email, email as label, hash, is_superuser,
			case when users.avatar is not null then
				json_build_object('path', users.avatar)
			else null end as avatar
			from users_users as users
			order by id offset $1::INT limit $2::INT
		) data";
		match lpsql._exec(query, prms) {
			Err(e) => println!("SQL err: {e}"),
			Ok(resp) => {
				for u in resp {
					r.push(serde_json::from_str(&u).unwrap());
				}
			}
		}
		return r
	}
	pub fn create(data: UserForm) -> Option<i32> {
		if validator::validate_email(&data.email) == false { return None::<i32> }
		let argon2 = Argon2::default();
		let salt = SaltString::generate(&mut OsRng);
		let hash = argon2.hash_password(&data.pwd.unwrap().into_bytes(), &salt).unwrap().to_string();
		let prms: Vec<qp> = vec![
			qp::String(data.email.to_string()),
			qp::String(hash.to_string()),
		];
		let query = "insert into users_users (email, hash) values ($1::TEXT, $2::TEXT) returning id";
		match lpsql.get_one(query, prms) {
			None => return None::<i32>,
			Some(id) => {
				return Some(id.parse().unwrap())
			}
		};

	}
	pub fn create_and_get(form: UserForm) -> Option<User> {
	  match UserDb::create(form) {
		None => return None,
		Some(id) => {
		  return UserDb::by_id(id);
		}
	  }
	}

	pub async fn update(id: i32, data: UserForm) -> Option<i32> {
		let prms: Vec<qp> = vec![
			qp::Number(id),
			qp::String(data.email),
			qp::Bool(data.is_superuser.unwrap()),
		];
		let q = "update users_users set email = $2::TEXT, is_superuser = $3::BOOL 
			where id = $1::INT
			returning id";
		match lpsql.get_one(q, prms) {
			None => None::<i32>,
			Some(id) => Some(id.parse().unwrap())
		}
		//User::update_avatar(id, data.avatar).await
	}

	pub async fn update_and_get(id: i32, data: UserForm) -> Option<User> {
	  match UserDb::update(id, data).await {
		None => return None,
		Some(id) => return UserDb::by_id(id),
	  }
	}
}
