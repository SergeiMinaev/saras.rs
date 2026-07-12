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

pub const SORTABLE_FIELDS: &[&str] = &["id", "email", "name", "is_superuser", "created_at"];
const DEFAULT_ORDER: &str = "id";

// Колонки поиска по подстроке. Добавить поле в поиск = дополнить список.
const SEARCH_COLS: &[&str] = &["users.name", "users.email"];

// JSON-проекция пользователя. Раньше дублировалась в каждой выборке — теперь
// объявлена один раз; набор и порядок полей меняются только здесь.
const USER_COLS: &str = "\
	users.id, users.email, users.email as label, users.name, users.hash, users.is_superuser,
	case when users.avatar is not null
		then json_build_object('path', users.avatar) else null end as avatar,
	case when users.default_avatar is not null
		then json_build_object('path', users.default_avatar) else null end as default_avatar,
	to_char(users.created_at at time zone 'UTC', 'YYYY-MM-DD\"T\"HH24:MI:SS\"Z\"') as created_at,
	to_char(users.updated_at at time zone 'UTC', 'YYYY-MM-DD\"T\"HH24:MI:SS\"Z\"') as updated_at";

// Начало любой выборки пользователя: обёртка row_to_json + проекция + from.
// Хвост (join/where/order/limit) и закрывающую ") data" дописывает вызывающий код.
fn select_users() -> Lpsql {
	let mut q = Lpsql::builder();
	q.push("select row_to_json(data) from ( select ")
		.push(USER_COLS)
		.push(" from users_users as users");
	q
}

// Условие поиска, если запрос задан. Значение (%term%) идёт только в push_bind;
// в format! попадают лишь имена колонок из SEARCH_COLS.
fn apply_search(q: &mut Lpsql, search: Option<&str>) {
	let Some(term) = search else { return };
	q.push(" where (");
	for (i, col) in SEARCH_COLS.iter().enumerate() {
		if i > 0 {
			q.push(" or ");
		}
		q.push(&format!("coalesce({col}, '') ilike "));
		q.push_bind(format!("%{term}%"));
		q.push("::text");
	}
	q.push(")");
}

impl UserDb {
	pub fn new(pool: Arc<ConnectionPool>) -> Self {
		UserDb { pool }
	}
	pub async fn sleep(&self) {
		let q = "select pg_sleep(3)";
		Lpsql::query(q).exec(&self.pool).await;
	}
	pub async fn total_count(&self, search: Option<&str>) -> i32 {
		let mut q = Lpsql::builder();
		q.push("select count(*) from users_users as users");
		apply_search(&mut q, search);
		q.fetch_one(&self.pool).await.unwrap().parse().unwrap()
	}
	pub async fn by_session_id(&self, sess_id: &str) -> Option<User> {
		let mut q = select_users();
		q.push(" join auth_sessions as session on users.id = session.user_id where session.id = ")
			.push_bind(sess_id)
			.push("::bytea and session.expires > now() ) data");
		q.fetch_one(&self.pool).await
			.and_then(|v| serde_json::from_str(&v).ok())
	}
	pub async fn by_id(&self, id: i32) -> Option<User> {
		let mut q = select_users();
		q.push(" where users.id = ").push_bind(id).push("::int ) data");
		q.fetch_one(&self.pool).await
			.and_then(|v| serde_json::from_str(&v).ok())
	}
	pub async fn by_email(&self, email: &str) -> Option<User> {
		let mut q = select_users();
		q.push(" where lower(btrim(users.email)) = ")
			.push_bind(normalize_email(email))
			.push("::text ) data");
		q.fetch_one(&self.pool).await
			.and_then(|v| serde_json::from_str(&v).ok())
	}
	pub async fn page(
		&self,
		offset: i32,
		size: i32,
		search: Option<&str>,
		sort_by: Option<&str>,
		sort_dir: Option<&str>,
	) -> Vec<User> {
		let order_clause = crate::admin::sort::order_by_clause(sort_by, sort_dir, SORTABLE_FIELDS, DEFAULT_ORDER);
		let mut q = select_users();
		apply_search(&mut q, search);
		q.push(&format!(" {order_clause} offset "))
			.push_bind(offset)
			.push("::int limit ")
			.push_bind(size)
			.push("::int ) data");
		let items = q.fetch_all(&self.pool).await;
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
			let q = "update users_users set name = $2::TEXT, updated_at = now()
				where id = $1::INT
				returning id";
			let _ = Lpsql::query(q).bind(id).bind(data.name.unwrap())
				.fetch_one(&self.pool).await;
		}
		let q = "update users_users set email = $2::TEXT, is_superuser = $3::BOOL, updated_at = now()
			where id = $1::INT
			returning id";
		Lpsql::query(q).bind(id).bind(email).bind(data.is_superuser.unwrap())
			.fetch_one(&self.pool).await
			.map(|id| id.parse().unwrap())
	}
	pub async fn set_password(&self, id: i32, pwd: &str) -> bool {
		let hash = hashing::hash_pwd(pwd);
		let q = "update users_users set hash = $2::TEXT, updated_at = now()
			where id = $1::INT returning id";
		Lpsql::query(q).bind(id).bind(hash).fetch_one(&self.pool).await.is_some()
	}
	pub async fn update_and_get(&self, id: i32, data: UserForm) -> Option<User> {
		let id = self.update(id, data).await?;
		self.by_id(id).await
	}
	pub async fn touch_last_visit(&self, user_id: u32) {
		let q = "update users_users set last_visit_at = now() where id = $1::INT";
		Lpsql::query(q).bind(user_id as i32).exec(&self.pool).await;
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
