use lpsql::Lpsql;
use lpsql::Pool;

#[derive(Clone, Debug)]
pub struct PasswordResetEntry {
	pub id: i64,
	pub user_id: i32,
	pub email: String,
	pub token_hash: String,
	pub expires_at: String,
	pub used_at: Option<String>,
}

pub struct PasswordResetDb {
	pub pool: Pool,
}

impl PasswordResetDb {
	pub fn new(pool: Pool) -> Self {
		Self { pool }
	}

	pub async fn is_in_cooldown(&self, email: &str, cooldown_sec: i64) -> bool {
		let q = "select 1 from auth_password_reset \
			where email = $1::TEXT and created_at > (now() - ($2::INT * interval '1 second')) \
			limit 1";
		Lpsql::query(q)
			.bind(email)
			.bind(cooldown_sec as i32)
			.fetch_one(&self.pool)
			.await
			.is_some()
	}

	pub async fn insert(
		&self,
		user_id: i32,
		email: &str,
		token_hash: &str,
		expires_at: &str,
		ip: Option<&str>,
		user_agent: Option<&str>,
	) -> bool {
		let q = "insert into auth_password_reset \
			(user_id, email, token_hash, expires_at, ip, user_agent) \
			values ($1::INT, $2::TEXT, $3::TEXT, $4::TIMESTAMPTZ, $5::TEXT, $6::TEXT) \
			returning id";
		Lpsql::query(q)
			.bind(user_id)
			.bind(email)
			.bind(token_hash)
			.bind(expires_at)
			.bind(ip.unwrap_or(""))
			.bind(user_agent.unwrap_or(""))
			.fetch_one(&self.pool)
			.await
			.is_some()
	}

	pub async fn by_token_hash(&self, token_hash: &str) -> Option<PasswordResetEntry> {
		let q = "select id, user_id, email, token_hash, expires_at, used_at \
			from auth_password_reset where token_hash = $1::TEXT limit 1";
		Lpsql::query(q)
			.bind(token_hash)
			.fetch_one(&self.pool)
			.await
			.map(|row| PasswordResetEntry {
				id: row.get("id").unwrap().parse().unwrap(),
				user_id: row.get("user_id").unwrap().parse().unwrap(),
				email: row.get("email").unwrap(),
				token_hash: row.get("token_hash").unwrap(),
				expires_at: row.get("expires_at").unwrap(),
				used_at: row.get("used_at"),
			})
	}

	pub async fn mark_used(&self, id: i64) -> bool {
		let q = "update auth_password_reset set used_at = now() where id = $1::BIGINT";
		Lpsql::query(q).bind(id).exec(&self.pool).await > 0
	}
}
