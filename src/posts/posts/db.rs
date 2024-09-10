use crate::lpsql::QueryParam as qp;
use crate::lpsql::Lpsql;
use crate::posts::posts::models::Post;
use crate::posts::posts::forms::PostForm;
use once_cell::sync::Lazy;
use std::sync::RwLock;
use serde::Deserialize;


pub static lpsql: Lazy<Lpsql> = Lazy::new(|| {
    Lpsql::new(None)
});


pub struct PostDb {}


impl PostDb {
	pub fn by_id(id: i32) -> Option<Post> {
		let prms: Vec<qp> = vec![
			qp::Number(id)
		];
		let query = "select row_to_json(data) from (
			select id, title as label, title, text
			from posts_posts as art where id = $1::INT
		) data";
		match lpsql.get_one(query, prms) {
			None => None::<Post>,
			Some(v) => {
				serde_json::from_str(&v).unwrap()
			}
		}
	}
	pub fn total_count() -> i32 {
	  let q = "select count(*) from posts_posts";
	  let p: Vec<qp> = vec![];
	  lpsql.get_one(q, p).unwrap().parse().unwrap()
	}
	pub fn page(offset: i32, size: i32) -> Vec<Post> {
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
		match lpsql._exec(query, prms) {
			Err(e) => println!("SQL err: {e}"),
			Ok(resp) => {
				for u in resp {
					r.push(serde_json::from_str(&u).unwrap());
				}
			}
		}
		return r
	}
	pub fn create(data: PostForm) -> Option<i32> {
		let prms: Vec<qp> = vec![
			qp::String(data.title.to_string()),
			qp::String(data.text.to_string()),
		];
		let query = "insert into posts_posts (title, text)
			values ($1::TEXT, $2::TEXT) returning id";
		match lpsql.get_one(query, prms) {
			None => return None::<i32>,
			Some(id) => {
				return Some(id.parse().unwrap())
			}
		};

	}
	pub fn create_and_get(form: PostForm) -> Option<Post> {
	  match PostDb::create(form) {
		None => return None,
		Some(id) => {
		  return PostDb::by_id(id);
		}
	  }
	}
	pub async fn update(id: i32, data: PostForm) -> Option<i32> {
		let prms: Vec<qp> = vec![
			qp::Number(id),
			qp::String(data.title),
			qp::String(data.text),
		];
		let q = "update posts_posts set title = $2::TEXT,
			text = $3::TEXT
			where id = $1::INT
			returning id";
		match lpsql.get_one(q, prms) {
			None => None::<i32>,
			Some(id) => Some(id.parse().unwrap())
		}
	}
	pub async fn update_and_get(id: i32, data: PostForm) -> Option<Post> {
	  match PostDb::update(id, data).await {
		None => return None,
		Some(id) => return PostDb::by_id(id),
	  }
	}
	pub fn delete(id: i32) -> bool {
		let prms: Vec<qp> = vec![qp::Number(id)];
		let q = "delete from posts_posts where id = $1::INT";
		lpsql.exec(q, prms)
	}
}
