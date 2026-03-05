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
		let query = "select
			case when avatar is not null then
				json_build_object('path', avatar)
			else null end as avatar
			from users_users where id = $1::INT";
		let a = Lpsql::query(query).bind(user_id).fetch_one(&self.pool).await.ok_or(())?;
		if a.is_empty() {
			return Err(());
		}
		serde_json::from_str(&a).map_err(|_| ())
	}
	pub async fn default_by_user_id(&self, user_id: i32) -> Result<Avatar, ()> {
		let query = "select
			case when default_avatar is not null then
				json_build_object('path', default_avatar)
			else null end as default_avatar
			from users_users where id = $1::INT";
		let a = Lpsql::query(query).bind(user_id).fetch_one(&self.pool).await.ok_or(())?;
		if a.is_empty() {
			return Err(());
		}
		serde_json::from_str(&a).map_err(|_| ())
	}
	pub async fn save(&self, user_id: i32, rel_path: &str) -> Result<(), Error> {
		let q = "update users_users set avatar = $2::TEXT, updated_at = now()
			where id = $1::INT";
		let r = Lpsql::query(q).bind(user_id).bind(rel_path).exec(&self.pool).await;
		if r == 1 {
			Ok(())
		} else {
			Err(Error::Database)
		}
	}
	pub async fn save_default(&self, user_id: i32, rel_path: &str) -> Result<(), Error> {
		let q = "update users_users set default_avatar = $2::TEXT, updated_at = now()
			where id = $1::INT";
		let r = Lpsql::query(q).bind(user_id).bind(rel_path).exec(&self.pool).await;
		if r == 1 {
			Ok(())
		} else {
			Err(Error::Database)
		}
	}
	pub async fn delete(&self, user_id: i32) -> bool {
		let q = "update users_users set avatar = null, updated_at = now()
			where id = $1::INT";
		Lpsql::query(q).bind(user_id).exec(&self.pool).await != 0
	}
	pub async fn delete_default(&self, user_id: i32) -> bool {
		let q = "update users_users set default_avatar = null, updated_at = now()
			where id = $1::INT";
		Lpsql::query(q).bind(user_id).exec(&self.pool).await != 0
	}
}
