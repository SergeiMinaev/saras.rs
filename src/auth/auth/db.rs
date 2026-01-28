use std::sync::Arc;
use lpsql::pool::ConnectionPool;
use lpsql::Lpsql;
use crate::auth::auth::models::Session;


pub struct AuthDb {
	pool: Arc<ConnectionPool>,
}

impl AuthDb {
	pub fn new(pool: Arc<ConnectionPool>) -> Self {
		AuthDb { pool }
	}
	pub async fn by_id(&self, id: String) -> Option<Session> {
		let query = "select row_to_json(data) from (\
			select id, expires, user_id from auth_sessions where id = $1::BYTEA \
		) data";
		Lpsql::query(query).bind(id).fetch_one(&self.pool).await
			.and_then(|v| serde_json::from_str(&v).ok())
	}
	pub async fn add_session(&self, id: Option<u32>) -> Option<Session> {
		let sess_id = match id {
			Some(id) => {
				let q = "insert into auth_sessions (user_id) values ($1::INT) returning id";
				Lpsql::query(q).bind(id).fetch_one(&self.pool).await.unwrap()
			},
			None => {
				let q = "insert into auth_sessions (user_id) values (null) returning id";
				Lpsql::query(q).fetch_one(&self.pool).await.unwrap()
			},
		};
		self.by_id(sess_id).await
	}

	pub async fn refresh_if_needed(&self, sess_id: &str, ttl_days: i64, min_remaining_days: i64) -> bool {
		let q = "update auth_sessions \
			set expires = now() + ($2::INT || ' days')::interval \
			where id = $1::BYTEA \
			and expires > now() \
			and expires < now() + ($3::INT || ' days')::interval";
		let updated = Lpsql::query(q)
			.bind(sess_id)
			.bind(ttl_days)
			.bind(min_remaining_days)
			.exec(&self.pool)
			.await;
		updated > 0
	}
}
