use lpsql::pool::ConnectionPool;
use lpsql::Lpsql;
use std::sync::Arc;
use crate::users::profile::models::Profile;



pub struct ProfileDb {
	pool: Arc<ConnectionPool>,
}

impl ProfileDb {
	pub fn new(pool: Arc<ConnectionPool>) -> Self {
		ProfileDb { pool }
	}
	pub async fn set_name(&self, id: u32, name: &str) -> bool {
		let q = "update users_users set name = $2::TEXT, updated_at = now()
			where id = $1::INT returning id";
		Lpsql::query(q).bind(id).bind(name).exec(&self.pool).await != 0
	}
	pub async fn by_id(&self, id: i32) -> Option<Profile> {
		let q = "select row_to_json(data) from (
			select name as label, name,
			case when users.avatar is not null then
				json_build_object('path', users.avatar)
			else null end as avatar,
			case when users.default_avatar is not null then
				json_build_object('path', users.default_avatar)
			else null end as default_avatar
			from users_users as users where id = $1::INT
		) data";
		Lpsql::query(q).bind(id).fetch_one(&self.pool).await
			.and_then(|v| serde_json::from_str(&v).ok())
	}
	pub async fn by_name(&self, name: &str) -> Option<Profile> {
		let q = "select row_to_json(data) from (
			select name as label, name,
			case when users.avatar is not null then
				json_build_object('path', users.avatar)
			else null end as avatar,
			case when users.default_avatar is not null then
				json_build_object('path', users.default_avatar)
			else null end as default_avatar
			from users_users as users where lower(name) = lower($1::TEXT)
		) data";
		Lpsql::query(q).bind(name).fetch_one(&self.pool).await
			.and_then(|v| serde_json::from_str(&v).ok())
	}
}
