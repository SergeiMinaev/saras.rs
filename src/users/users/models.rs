use argon2::{
	password_hash::{
		PasswordHash, PasswordVerifier
	},
	Argon2
};
use std::path::PathBuf;
use serde::{Serialize,Deserialize};
use lpsql::QueryParam as qp;
use crate::auth::sessions::Session;
use crate::models::base_model::BaseModel;
use crate::models::base::{ ImageStorage };
use crate::models::image_field::{ ImageField };
use crate::db::get_pool;
use lpsql::pool::ConnectionPool;
use std::sync::Arc;


//pub static lpsql: Lazy<Lpsql> = Lazy::new(|| {
//    Lpsql::new(None)
//});



const AVATAR_UPLOAD_TO: &'static str = "users/avatars";


#[derive(Serialize, Deserialize, Debug, schemars::JsonSchema)]
#[serde(rename_all = "lowercase")]
pub struct TextField {
	#[validate(length(max = 1000))]
	text: String
}

#[derive(Serialize, Deserialize, Debug, schemars::JsonSchema)]
#[serde(rename_all = "lowercase")]
pub struct User {
	pub id: u32,
	#[schemars(title = "Email", description = "Email пользователя")]
	pub email: String,
	//#[serde(skip_deserializing)]
	pub hash: String,
	pub is_superuser: bool,
	pub avatar: Option<ImageField>,
	pub qwe: Option<TextField>,
}

impl BaseModel for User {
	const NAME: &'static str = "Пользователь";
	const NAME_PLURAL: &'static str = "Пользователи";
}

impl User {
	pub fn is_logged_with_oauth(&self) -> bool {
		return self.hash == "".to_string()
	}
	pub async fn update_avatar(id: i32, avatar: Option<String>) -> bool {
		match avatar {
			None => return true,
			Some(avatar) => {
				// Empty string means deletion.
				let prms: Vec<qp> = vec![qp::Number(id)];
				let q = "select avatar from users_users where id = $1::INT";
				let pool: Arc<ConnectionPool> = get_pool();
				let pool = pool.clone();
				let conn = pool.get_conn().await;
				//let conn = {
				//	let mut pool_lock = pool.lock().await;
				//	pool_lock.get_conn().await
				//};
				let existing_path: String = conn.get_one(q, prms).await.unwrap();
				println!("existing path? {existing_path}");
				if existing_path != "" {
					ImageStorage::delete(&existing_path).await;
				}
				if avatar == "" {
					if existing_path != "" {
						let prms: Vec<qp> = vec![qp::Number(id)];
						let q = "update users_users set avatar = null where id = $1::INT";
						let _result = conn.exec(q, prms).await;
						//{
						//	let mut pool_lock = pool.lock().await;
						//	pool_lock.release_conn(conn).await;
						//}
		pool.release_conn(conn).await;
						return true
					} else {
						//{
						//	let mut pool_lock = pool.lock().await;
						//	pool_lock.release_conn(conn).await;
						//}
		pool.release_conn(conn).await;
						return true
					}
				} else {
					let mut path: PathBuf = ImageStorage::save_from_base64(
							&avatar, AVATAR_UPLOAD_TO).await;
					let stem = format!("{}", path.file_stem().unwrap().to_str().unwrap());
					path.set_file_name(stem);
					let prms: Vec<qp> = vec![
						qp::Number(id),
						qp::String(path.to_string_lossy().to_string())
					];
					let q = "update users_users set avatar = $2::TEXT where id = $1::INT";
					let _result = conn.exec(q, prms).await;
					//{
					//	let mut pool_lock = pool.lock().await;
					//	pool_lock.release_conn(conn).await;
					//}
		pool.release_conn(conn).await;
					return true
				}
			}
		}
	}
	pub async fn delete(id: i32) -> bool {
	   let prms: Vec<qp> = vec![qp::Number(id)];
	   let q = "delete from users_users where id = $1::INT";
				let pool = get_pool();
				let pool = pool.clone();
				//let conn = {
				//	let mut pool_lock = pool.lock().await;
				//	pool_lock.get_conn().await
				//};
				let conn = pool.get_conn().await;
	   let _result = conn.exec(q, prms).await;
					//{
					//	let mut pool_lock = pool.lock().await;
					//	pool_lock.release_conn(conn).await;
					//}
		pool.release_conn(conn).await;
	   return true
	}
	pub async fn by_email(email: String) -> Option<User> {
		println!("by_email {email}");
		let prms: Vec<qp> = vec![
			qp::String(email)
		];
		let query = "select row_to_json(data) from (\
			select id, email, hash, is_superuser from users_users where email = $1::TEXT \
		) data";
				let pool = get_pool();
				//let pool = pool.clone();
				//let conn = {
				//	let mut pool_lock = pool.lock().await;
				//	pool_lock.get_conn().await
				//};
				let conn = pool.get_conn().await;
		let result = match conn.get_one(query, prms).await {
			None => None::<User>,
			Some(v) => {
				serde_json::from_str(&v).unwrap()
			}
		};
		println!("got result: {result:?}");
					//{
					//	let mut pool_lock = pool.lock().await;
					//	pool_lock.release_conn(conn).await;
					//}
		pool.release_conn(conn).await;
		result
	}
	pub async fn by_session_id(sess_id: &String) -> Option<User> {
		let prms: Vec<qp> = vec![
			qp::String(sess_id.to_string())
		];
		let query = "select row_to_json(data) from (\
			select usr.id, email, hash, is_superuser \
			from users_users as usr join auth_sessions as session \
			on usr.id = session.user_id where session.id = $1::BYTEA \
			and session.expires > now()
		) data";
		let pool = get_pool();
		let conn = pool.get_conn().await;
		//let conn = {
		//	let mut pool_lock = pool.lock().await;
		//	pool_lock.get_conn().await
		//};
		let result = match conn.get_one(query, prms).await {
			None => None::<User>,
			Some(v) => {
				serde_json::from_str(&v).unwrap()
			}
		};
		//{
		//	let mut pool_lock = pool.lock().await;
		//	pool_lock.release_conn(conn).await;
		//}
		pool.release_conn(conn).await;
		result
	}
	pub fn check_password(&self, pwd: String) -> bool {
		match PasswordHash::new(&self.hash) {
			Err(e) => {
				println!("Unable to create PasswordHash: {e:?}");
				return false
			},
			Ok(hash) => {
				match Argon2::default().verify_password(pwd.as_bytes(), &hash) {
					Err(_) => return false,
					Ok(()) => return true,
				}
			},
		}
	}
	pub async fn add_session(&self) -> Option<Session> {
		let prms: Vec<qp> = vec![
			qp::String(self.id.to_string())
		];
		let query = "insert into auth_sessions (user_id) values ($1::INT) returning id";
				let pool = get_pool();
				//let conn = {
				//	let mut pool_lock = pool.lock().await;
				//	pool_lock.get_conn().await
				//};
				let conn = pool.get_conn().await;
		let result = match conn.get_one(query, prms).await {
			None => None::<Session>,
			Some(id) => {
				Session::by_id(id).await
			}
		};
					//{
					//	let mut pool_lock = pool.lock().await;
					//	pool_lock.release_conn(conn).await;
					//}
					pool.release_conn(conn).await;
		result
	}
	pub async fn all() -> Vec<User> {
		let mut r: Vec<User> = vec![];
		let prms: Vec<qp> = vec![];
		let query = "select row_to_json(data) from (\
			select id, email, hash, is_superuser from users\
		) data";
				let pool = get_pool();
				//let conn = {
				//	let mut pool_lock = pool.lock().await;
				//	pool_lock.get_conn().await
				//};
				let conn = pool.get_conn().await;
		match conn.exec(query, prms).await {
			Err(e) => println!("ERR: {e}"),
			Ok(resp) => {
				for u in resp {
					r.push(serde_json::from_str(&u).unwrap());
				}
			}
		}
					//{
					//	let mut pool_lock = pool.lock().await;
					//	pool_lock.release_conn(conn).await;
					//}
					pool.release_conn(conn).await;
		return r
	}
}
