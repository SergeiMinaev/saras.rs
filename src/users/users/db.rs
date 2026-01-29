use lpsql::Lpsql;
use crate::users::users::models::User;
use crate::users::users::forms::UserForm;
use argon2::{
	password_hash::{
		rand_core::OsRng,
		PasswordHasher, SaltString
	},
	Argon2
};
use lpsql::pool::ConnectionPool;
use crate::util::{random_string, normalize_email};
use crate::auth::auth::hashing;
use std::sync::Arc;
use validator::ValidateEmail;


pub struct UserDb {
	pool: Arc<ConnectionPool>,
}

impl UserDb {
	pub fn new(pool: Arc<ConnectionPool>) -> Self {
		UserDb { pool }
	}
	pub async fn sleep(&self) {
		let q = "select pg_sleep(3)";
		Lpsql::query(q).exec(&self.pool).await;
	}
	pub async fn total_count(&self) -> i32 {
		let q = "select count(*) from users_users";
		Lpsql::query(q).fetch_one(&self.pool).await.unwrap().parse().unwrap()
	}
	pub async fn total_count_by_name(&self, query: &str) -> i32 {
		let q = "select count(*)
			from users_users
			where coalesce(name, '') ilike $1::TEXT";
		let pattern = format!("%{}%", query);
		Lpsql::query(q)
			.bind(pattern)
			.fetch_one(&self.pool)
			.await
			.unwrap()
			.parse()
			.unwrap()
	}
	pub async fn by_session_id(&self, sess_id: &str) -> Option<User> {
		let q = "select row_to_json(data) from (\
			select users.id, email as label, email, name, hash, is_superuser,
			case when users.avatar is not null then
				json_build_object('path', users.avatar)
			else null end as avatar,
			case when users.default_avatar is not null then
				json_build_object('path', users.default_avatar)
			else null end as default_avatar
			from users_users as users join auth_sessions as session \
			on users.id = session.user_id where session.id = $1::BYTEA \
			and session.expires > now()
		) data";
		Lpsql::query(q).bind(sess_id).fetch_one(&self.pool).await
			.and_then(|v| serde_json::from_str(&v).ok())
	}
	pub async fn by_id(&self, id: i32) -> Option<User> {
		let q = "select row_to_json(data) from (
			select id, email as label, email, name, hash, is_superuser,
			case when users.avatar is not null then
				json_build_object('path', users.avatar)
			else null end as avatar,
			case when users.default_avatar is not null then
				json_build_object('path', users.default_avatar)
			else null end as default_avatar
			from users_users as users where id = $1::INT
		) data";
		Lpsql::query(q).bind(id).fetch_one(&self.pool).await
			.and_then(|v| serde_json::from_str(&v).ok())
	}
	pub async fn by_email(&self, email: &str) -> Option<User> {
		let email = normalize_email(email);
		let q = "select row_to_json(data) from (
			select id, email as label, email, name, hash, is_superuser,
			case when users.avatar is not null then
				json_build_object('path', users.avatar)
			else null end as avatar,
			case when users.default_avatar is not null then
				json_build_object('path', users.default_avatar)
			else null end as default_avatar
			from users_users as users where lower(email) = $1::TEXT
		) data";
		Lpsql::query(q).bind(email).fetch_one(&self.pool).await
			.and_then(|v| serde_json::from_str(&v).ok())
	}
	pub async fn page(&self, offset: i32, size: i32) -> Vec<User> {
		let q = "select row_to_json(data) from (
			select id, email, email as label, name, hash, is_superuser,
			case when users.avatar is not null then
				json_build_object('path', users.avatar)
			else null end as avatar,
			case when users.default_avatar is not null then
				json_build_object('path', users.default_avatar)
			else null end as default_avatar
			from users_users as users
			order by id offset $1::INT limit $2::INT
		) data";
		let items = Lpsql::query(q).bind(offset).bind(size).fetch_all(&self.pool).await;
		items.into_iter().map(|json| serde_json::from_str(&json).unwrap()).collect()
	}
	pub async fn page_by_name(&self, offset: i32, size: i32, query: &str) -> Vec<User> {
		let q = "select row_to_json(data) from (
			select id, email, email as label, name, hash, is_superuser,
			case when users.avatar is not null then
				json_build_object('path', users.avatar)
			else null end as avatar,
			case when users.default_avatar is not null then
				json_build_object('path', users.default_avatar)
			else null end as default_avatar
			from users_users as users
			where coalesce(name, '') ilike $3::TEXT
			order by id offset $1::INT limit $2::INT
		) data";
		let pattern = format!("%{}%", query);
		let items = Lpsql::query(q)
			.bind(offset)
			.bind(size)
			.bind(pattern)
			.fetch_all(&self.pool)
			.await;
		items.into_iter().map(|json| serde_json::from_str(&json).unwrap()).collect()
	}
	pub async fn create(&self, data: UserForm) -> Option<i32> {
		if !data.email.validate_email() { return None::<i32> }
		let email = normalize_email(&data.email);
		let argon2 = Argon2::default();
		let salt = SaltString::generate(&mut OsRng);
		let pwd = data.pwd.clone().unwrap_or_else(|| random_string(32));
		let hash = argon2.hash_password(&pwd.into_bytes(), &salt).unwrap().to_string();
		if let Some(name) = data.name {
			let q = "insert into users_users (email, name, hash)
				values ($1::TEXT, $2::TEXT, $3::TEXT) returning id";
			Lpsql::query(q).bind(email).bind(name).bind(hash)
				.fetch_one(&self.pool).await
				.map(|id| id.parse().unwrap())
		} else {
			let q = "insert into users_users (email, hash)
				values ($1::TEXT, $2::TEXT) returning id";
			Lpsql::query(q).bind(email).bind(hash)
				.fetch_one(&self.pool).await
				.map(|id| id.parse().unwrap())
		}
	}
	pub async fn create_and_get(&self, form: UserForm) -> Option<User> {
		let id = self.create(form).await?;
		self.by_id(id).await
	}
	pub async fn update(&self, id: i32, data: UserForm) -> Option<i32> {
		let email = normalize_email(&data.email);
		if data.name.is_some() {
			let q = "update users_users set name = $2::TEXT
				where id = $1::INT
				returning id";
			let _ = Lpsql::query(q).bind(id).bind(data.name.unwrap())
				.fetch_one(&self.pool).await;
		}
		let q = "update users_users set email = $2::TEXT, is_superuser = $3::BOOL 
			where id = $1::INT
			returning id";
		Lpsql::query(q).bind(id).bind(email).bind(data.is_superuser.unwrap())
			.fetch_one(&self.pool).await
			.map(|id| id.parse().unwrap())
	}
	pub async fn set_password(&self, id: i32, pwd: &str) -> bool {
		let hash = hashing::hash_pwd(pwd);
		let q = "update users_users set hash = $2::TEXT where id = $1::INT returning id";
		Lpsql::query(q).bind(id).bind(hash).fetch_one(&self.pool).await.is_some()
	}
	pub async fn update_and_get(&self, id: i32, data: UserForm) -> Option<User> {
		let id = self.update(id, data).await?;
		self.by_id(id).await
	}
	pub async fn delete(&self, id: i32) -> bool {
		let q = "delete from users_users where id = $1::INT";
		Lpsql::query(q).bind(id).exec(&self.pool).await != 0
	}
	pub async fn hash(&self, id: u32) -> String {
		let q = "select hash from users_users where id = $1::INT";
		Lpsql::query(q).bind(id).fetch_one(&self.pool).await.unwrap()
	}
}
