use crate::users::avatars::models::Avatar;
use crate::errors::Error;
use lpsql::pool::ConnectionPool;
use lpsql::Lpsql;
use std::sync::Arc;



pub struct AvatarDb {
	pool: Arc<ConnectionPool>,
}

impl AvatarDb {
	pub fn new(pool: Arc<ConnectionPool>) -> Self {
		AvatarDb { pool }
	}
	pub async fn by_user_id(&self, user_id: i32) -> Result<Avatar, ()> {
		let query = "select avatar from users_users where id = $1::INT";
		let a: Avatar = Lpsql::query(query).bind(user_id).fetch_one(&self.pool).await
			.and_then(|v| serde_json::from_str(&v).ok()).unwrap();
		return Ok(a)
	}
	pub async fn save(&self, user_id: i32, rel_path: &str) -> Result<(), Error> {
		let q = "update users_users set avatar = $2::TEXT where id = $1::INT";
		let r = Lpsql::query(q).bind(user_id).bind(rel_path).exec(&self.pool).await;
		if r == 1 {
			Ok(())
		} else {
			Err(Error::Database)
		}
	}
	pub async fn delete(&self, user_id: i32) -> bool {
		let q = "update users_users set avatar = null where id = $1::INT";
		Lpsql::query(q).bind(user_id).exec(&self.pool).await != 0
	}
}
