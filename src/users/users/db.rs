use lpsql::QueryParam as qp;
use crate::users::users::models::User;
use crate::users::users::forms::UserForm;
use argon2::{
	password_hash::{
		rand_core::OsRng,
		PasswordHasher, SaltString
	},
	Argon2
};
use lpsql::pool::ConnectionPool;
use std::sync::Arc;
use crate::errors::Error;
//use smol::Timer;
//use std::time::Duration;


pub struct UserDb {
	pool: Arc<ConnectionPool>,
}
//async fn async_sleep() {
//	Timer::after(Duration::from_secs(3)).await;
//}

impl UserDb {
	pub fn new(pool: Arc<ConnectionPool>) -> Self {
		UserDb { pool }
	}
	pub async fn sleep(&self) {
		let q = "select pg_sleep(3)";
		let p: Vec<qp> = vec![];
		let conn = self.pool.get_conn().await;
		let _ = conn.exec(q, p).await;
		self.pool.release_conn(conn).await;
	}
	pub async fn total_count(&self) -> i32 {
		let q = "select count(*) from users_users";
		let p: Vec<qp> = vec![];
		let conn = self.pool.get_conn().await;
		let result = conn.get_one(q, p).await.unwrap().parse().unwrap();
		self.pool.release_conn(conn).await;
		result
	}
	pub async fn by_id(&self, id: i32) -> Option<User> {
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
		let conn = self.pool.get_conn().await;
		let result = match conn.get_one(query, prms).await {
			None => None::<User>,
			Some(v) => {
				serde_json::from_str(&v).unwrap()
			}
		};
		self.pool.release_conn(conn).await;
		result
	}
	pub async fn page(&self, offset: i32, size: i32) -> Vec<User> {
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
		let conn = self.pool.get_conn().await;
		match conn.exec(query, prms).await {
			Err(e) => println!("SQL err: {e}"),
			Ok(resp) => {
				for u in resp {
					r.push(serde_json::from_str(&u).unwrap());
				}
			}
		}
		self.pool.release_conn(conn).await;
		return r
	}
	pub async fn create(&self, data: UserForm) -> Option<i32> {
		if validator::validate_email(&data.email) == false { return None::<i32> }
		let argon2 = Argon2::default();
		let salt = SaltString::generate(&mut OsRng);
		let hash = argon2.hash_password(&data.pwd.unwrap().into_bytes(), &salt).unwrap().to_string();
		let prms: Vec<qp> = vec![
			qp::String(data.email.to_string()),
			qp::String(hash.to_string()),
		];
		let query = "insert into users_users (email, hash) values ($1::TEXT, $2::TEXT) returning id";
		let conn = self.pool.get_conn().await;
		let _result = match conn.get_one(query, prms).await {
			None => {
				self.pool.release_conn(conn).await;
				return None::<i32>
			},
			Some(id) => {
				self.pool.release_conn(conn).await;
				return Some(id.parse().unwrap())
			}
		};

	}
	pub async fn create_and_get(&self, form: UserForm) -> Option<User> {
	  match self.create(form).await {
		None => return None,
		Some(id) => {
		  return self.by_id(id).await;
		}
	  }
	}

	pub async fn update(&self, id: i32, data: UserForm) -> Option<i32> {
		let prms: Vec<qp> = vec![
			qp::Number(id),
			qp::String(data.email),
			qp::Bool(data.is_superuser.unwrap()),
		];
		let q = "update users_users set email = $2::TEXT, is_superuser = $3::BOOL 
			where id = $1::INT
			returning id";
		let conn = self.pool.get_conn().await;
		let result = match conn.get_one(q, prms).await {
			None => None::<i32>,
			Some(id) => Some(id.parse().unwrap())
		};
		self.pool.release_conn(conn).await;
		result
	}

	pub async fn update_and_get(&self, id: i32, data: UserForm) -> Option<User> {
	  match self.update(id, data).await {
		None => return None,
		Some(id) => return self.by_id(id).await,
	  }
	}

	pub async fn delete(&self, id: i32) -> Result<(), Error> {
		let prms: Vec<qp> = vec![qp::Number(id)];
		let q = "delete from users_users where id = $1::INT";
		let conn = self.pool.get_conn().await;
		let rows_affected = conn.delete(q, prms).await.unwrap();
		self.pool.release_conn(conn).await;
		if rows_affected > 0 {
			Ok(())
		} else {
			Err(Error::Database)
		}
	}
}
