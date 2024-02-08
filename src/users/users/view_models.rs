use serde::{Serialize,Deserialize};
use lpsql::QueryParam as qp;
use crate::models::ImageField;


#[derive(Serialize, Deserialize, Debug, schemars::JsonSchema)]
#[serde(rename_all = "lowercase")]
pub struct UserView {
	pub id: u32,
	pub label: String,
	#[schemars(title = "Email", description = "Email пользователя")]
	pub email: String,
	pub is_superuser: bool,
	pub avatar: Option<ImageField>,
}


impl UserView {
	pub fn by_id(id: i32) -> Option<UserView> {
		let prms: Vec<qp> = vec![
			qp::Number(id)
		];
		let query = "select row_to_json(data) from (
			select id, email as label, email, hash, is_superuser,
			case when users.avatar is not null then
				json_build_object('path', users.avatar)
			else null end as avatar
			from users_users as users where id = $1::INT
		) data";
		match lpsql::get_one(query, prms) {
			None => None::<UserView>,
			Some(v) => {
				return serde_json::from_str(&v).unwrap();
			}
		};
		None
	}
	pub fn all() -> Vec<UserView> {
		let mut r: Vec<UserView> = vec![];
		let prms: Vec<qp> = vec![];
		let query = "select row_to_json(data) from (\
			select id, email, hash, is_superuser, email as label,
			case when users.avatar is not null then
				json_build_object('path', users.avatar)
			else null end as avatar
			from users_users
		) data";
		match lpsql::_exec(query, prms) {
			Err(e) => println!("ERR: {e}"),
			Ok(resp) => {
				for u in resp {
					r.push(serde_json::from_str(&u).unwrap());
				}
			}
		}
		return r
	}
	pub fn total_count() -> i32 {
	  let q = "select count(*) from users_users";
	  let p: Vec<qp> = vec![];
	  lpsql::get_one(q, p).unwrap().parse().unwrap()
	}
	pub fn page(offset: i32, size: i32) -> Vec<UserView> {
		//let total_count = UserView::total_count();
		let mut r: Vec<UserView> = vec![];
		let prms: Vec<qp> = vec![
		  qp::Number(offset),
		  qp::Number(size),
		];
		let query = "select row_to_json(data) from (
			select id, email, email as label, is_superuser,
			case when users.avatar is not null then
				json_build_object('path', users.avatar)
			else null end as avatar
			from users_users as users
			order by id offset $1::INT limit $2::INT
		) data";
		match lpsql::_exec(query, prms) {
			Err(e) => println!("ERR: {e}"),
			Ok(resp) => {
				for u in resp {
					r.push(serde_json::from_str(&u).unwrap());
				}
			}
		}
		return r
	}
}
