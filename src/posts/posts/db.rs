use crate::lpsql::QueryParam as qp;
use crate::posts::posts::models::Post;
use crate::posts::posts::forms::PostForm;
use lpsql::pool::ConnectionPool;
use std::sync::Arc;




pub struct PostDb {
	pool: Arc<ConnectionPool>,
}


impl PostDb {
	pub fn new(pool: Arc<ConnectionPool>) -> Self {
		PostDb { pool }
	}
	pub async fn page(&self, offset: i32, size: i32) -> Vec<Post> {
		let mut r: Vec<Post> = vec![];
		let prms: Vec<qp> = vec![
		  qp::Number(offset),
		  qp::Number(size),
		];
		let query = "select row_to_json(data) from (
			select id, title as label, title, text
			from posts_posts as art
			order by id offset $1::INT limit $2::INT
		) data";
		//let conn = {
		//	let mut pool_lock = self.pool.lock().await;
		//	pool_lock.get_conn().await
		//};
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
		//{
		//	let mut pool_lock = self.pool.lock().await;
		//	pool_lock.release_conn(conn).await;
		//}
		r
	}
	pub async fn by_id(&self, id: i32) -> Option<Post> {
		let prms: Vec<qp> = vec![
			qp::Number(id)
		];
		let query = "select row_to_json(data) from (
			select id, title as label, title, text
			from posts_posts as art where id = $1::INT
		) data";
		//let conn = {
		//	let mut pool_lock = self.pool.lock().await;
		//	pool_lock.get_conn().await
		//};
		let conn = self.pool.get_conn().await;
		let result = match conn.get_one(query, prms).await {
			None => None::<Post>,
			Some(v) => {
				serde_json::from_str(&v).unwrap()
			}
		};
		self.pool.release_conn(conn).await;
		//{
		//	let mut pool_lock = self.pool.lock().await;
		//	pool_lock.release_conn(conn).await;
		//}
		result
	}
	pub async fn total_count(&self) -> i32 {
		let q = "select count(*) from posts_posts";
		let p: Vec<qp> = vec![];
		//let conn = {
		//	let mut pool_lock = self.pool.lock().await;
		//	pool_lock.get_conn().await
		//};
		let conn = self.pool.get_conn().await;
		let count = conn.get_one(q, p).await.unwrap().parse().unwrap();
		//{
		//	let mut pool_lock = self.pool.lock().await;
		//	pool_lock.release_conn(conn).await;
		//}
		self.pool.release_conn(conn).await;
		count
	}
	pub async fn create(&self, data: PostForm) -> Option<i32> {
		let prms: Vec<qp> = vec![
			qp::String(data.title.to_string()),
			qp::String(data.text.to_string()),
		];
		let query = "insert into posts_posts (title, text)
			values ($1::TEXT, $2::TEXT) returning id";
		//let conn = {
		//	let mut pool_lock = self.pool.lock().await;
		//	pool_lock.get_conn().await
		//};
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
		//{
		//	let mut pool_lock = self.pool.lock().await;
		//	pool_lock.release_conn(conn).await;
		//}
		//result
	}
	pub async fn create_and_get(&self, form: PostForm) -> Option<Post> {
	  match self.create(form).await {
		None => return None,
		Some(id) => {
		  return self.by_id(id).await;
		}
	  }
	}
	pub async fn update(&self, id: i32, data: PostForm) -> Option<i32> {
		let prms: Vec<qp> = vec![
			qp::Number(id),
			qp::String(data.title),
			qp::String(data.text),
		];
		let q = "update posts_posts set title = $2::TEXT,
			text = $3::TEXT
			where id = $1::INT
			returning id";
		//let conn = {
		//	let mut pool_lock = self.pool.lock().await;
		//	pool_lock.get_conn().await
		//};
		let conn = self.pool.get_conn().await;
		let result = match conn.get_one(q, prms).await {
			None => None::<i32>,
			Some(id) => Some(id.parse().unwrap())
		};
		//{
		//	let mut pool_lock = self.pool.lock().await;
		//	pool_lock.release_conn(conn).await;
		//}
		self.pool.release_conn(conn).await;
		result
	}
	pub async fn update_and_get(&self, id: i32, data: PostForm) -> Option<Post> {
	  match self.update(id, data).await {
		None => return None,
		Some(id) => return self.by_id(id).await,
	  }
	}
	pub async fn delete(&self, id: i32) -> bool {
		let prms: Vec<qp> = vec![qp::Number(id)];
		let q = "delete from posts_posts where id = $1::INT";
		//let conn = {
		//	let mut pool_lock = self.pool.lock().await;
		//	pool_lock.get_conn().await
		//};
		let conn = self.pool.get_conn().await;
		let r = conn.exec(q, prms).await;
		println!("delete result: {r:?}");
		//{
		//	let mut pool_lock = self.pool.lock().await;
		//	pool_lock.release_conn(conn).await;
		//}
		self.pool.release_conn(conn).await;
		true
	}
}
