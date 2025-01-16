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
	pub async fn add_session(&self, id: u32) -> Option<Session> {
		let query = "insert into auth_sessions (user_id) values ($1::INT) returning id";
		let sess_id: String = Lpsql::query(query).bind(id).fetch_one(&self.pool).await.unwrap();
		self.by_id(sess_id).await
	}
}
