use crate::posts::posts::models::Post;
use crate::posts::posts::forms::PostForm;
use lpsql::pool::ConnectionPool;
use lpsql::Lpsql;
use std::sync::Arc;




pub struct PostDb {
	pool: Arc<ConnectionPool>,
}


pub const SORTABLE_FIELDS: &[&str] = &["id", "title"];
const DEFAULT_ORDER: &str = "id";

impl PostDb {
	pub fn new(pool: Arc<ConnectionPool>) -> Self {
		PostDb { pool }
	}
	pub async fn page(
		&self,
		offset: i32,
		size: i32,
		sort_by: Option<&str>,
		sort_dir: Option<&str>,
	) -> Vec<Post> {
		let order_clause = crate::admin::sort::order_by_clause(sort_by, sort_dir, SORTABLE_FIELDS, DEFAULT_ORDER);
		let q = format!("select row_to_json(data) from (
			select id, title as label, title, text
			from posts_posts as art
			{order_clause} offset $1::INT limit $2::INT
		) data");
		let items = Lpsql::query(&q).bind(offset).bind(size).fetch_all(&self.pool).await;
		items.into_iter().map(|json| serde_json::from_str(&json).unwrap()).collect()
	}
	pub async fn by_id(&self, id: i32) -> Option<Post> {
		let query = "select row_to_json(data) from (
			select id, title as label, title, text
			from posts_posts as art where id = $1::INT
		) data";
		Lpsql::query(query).bind(id).fetch_one(&self.pool).await
			.and_then(|v| serde_json::from_str(&v).ok())
	}
	pub async fn total_count(&self) -> i32 {
		let q = "select count(*) from posts_posts";
		Lpsql::query(q).fetch_one(&self.pool).await.unwrap().parse().unwrap()
	}
	pub async fn create(&self, data: PostForm) -> Option<i32> {
		let q = "insert into posts_posts (title, text)
			values ($1::TEXT, $2::TEXT) returning id";
		Lpsql::query(q).bind(data.title).bind(data.text)
			.fetch_one(&self.pool).await
			.map(|id| id.parse().unwrap())
	}
	pub async fn create_and_get(&self, form: PostForm) -> Option<Post> {
		let id = self.create(form).await?;
		self.by_id(id).await
	}
	pub async fn update(&self, id: i32, data: PostForm) -> Option<i32> {
		let q = "update posts_posts set title = $2::TEXT,
			text = $3::TEXT
			where id = $1::INT
			returning id";
		Lpsql::query(q).bind(id).bind(data.title).bind(data.text).fetch_one(&self.pool).await
			.map(|id| id.parse().unwrap())
	}
	pub async fn update_and_get(&self, id: i32, data: PostForm) -> Option<Post> {
		let id = self.update(id, data).await?;
		self.by_id(id).await
	}
	pub async fn delete(&self, id: i32) -> bool {
		let q = "delete from posts_posts where id = $1::INT";
		Lpsql::query(q).bind(id).exec(&self.pool).await != 0
	}
}
