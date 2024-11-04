use serde::{ Serialize,Deserialize };
use lpsql::QueryParam as qp;
use crate::db::get_pool;


#[derive(Serialize, Deserialize, Debug)]
pub struct Session {
	pub id: String,
	pub expires: String,
	pub user_id: u32,
}

impl Session {
	pub async fn by_id(id: String) -> Option<Session> {
		let prms: Vec<qp> = vec![
			qp::String(id)
		];
		let query = "select row_to_json(data) from (\
			select id, expires, user_id from auth_sessions where id = $1::BYTEA \
		) data";
		let pool = get_pool();
		let pool = pool.clone();
		let conn = pool.get_conn().await;
		let result = match conn.get_one(query, prms).await {
			None => None::<Session>,
			Some(v) => {
				return serde_json::from_str(&v).unwrap();
			}
		};
		pool.release_conn(conn).await;
		result
	}
}
