use lpsql::pool::ConnectionPool;
use lpsql::Lpsql;
use std::sync::Arc;



pub struct ProfileDb {
	pool: Arc<ConnectionPool>,
}

impl ProfileDb {
	pub fn new(pool: Arc<ConnectionPool>) -> Self {
		ProfileDb { pool }
	}
	pub async fn set_name(&self, id: u32, name: &str) -> bool {
		let q = "update users_users set name = $2::TEXT where id = $1::INT returning id";
		Lpsql::query(q).bind(id).bind(name).exec(&self.pool).await != 0
	}
}
