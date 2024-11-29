use lpsql::QueryParam as qp;
use crate::users::avatars::models::Avatar;
use crate::errors::Error;
use lpsql::pool::ConnectionPool;
use std::sync::Arc;
use log::debug;



pub struct AvatarDb {
	pool: Arc<ConnectionPool>,
}

impl AvatarDb {
	pub fn new(pool: Arc<ConnectionPool>) -> Self {
		AvatarDb { pool }
	}
	pub async fn by_user_id(&self, user_id: i32) -> Result<Avatar, ()> {
		let prms: Vec<qp> = vec![
			qp::Number(user_id)
		];
		let query = "select avatar from users_users where id = $1::INT";
		let conn = self.pool.get_conn().await;
		let result = match conn.get_one(query, prms).await {
			None => Err(()),
			Some(path) => {
				Ok(Avatar { path: path })
			}
		};
		self.pool.release_conn(conn).await;
		result
	}
	pub async fn save(&self, user_id: i32, rel_path: &str) -> Result<(), Error> {
		let prms: Vec<qp> = vec![
			qp::Number(user_id),
			qp::String(rel_path.to_string()),
		];
		let query = "update users_users set avatar = $2::TEXT where id = $1::INT";
		let conn = self.pool.get_conn().await;
		let _result = conn.exec(query, prms).await.map_err(|err| {
			panic!("AvatarDB::save failed: {err}");
			//Error::Database
		});
		self.pool.release_conn(conn).await;
		Ok(())
	}
	pub async fn delete(&self, user_id: i32) -> bool {
		let prms: Vec<qp> = vec![
			qp::Number(user_id)
		];
		let query = "update users_users set avatar = null where id = $1::INT"; //let conn = {
		let conn = self.pool.get_conn().await;
		
		conn.exec(query, prms).await.unwrap_or_else(|err| {
			//debug!("AvatarDb::delete failed with query: {query}, prms: {:?}", prms);
			panic!("AvatarDb::delete failed: {err:?}")
		});
		self.pool.release_conn(conn).await;
		true
	}
}
