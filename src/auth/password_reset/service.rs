use chrono::{Duration, Utc};
use rand::{RngCore, thread_rng};
use base64::engine::general_purpose::URL_SAFE_NO_PAD;
use base64::Engine;
use sha2::{Sha256, Digest};
use crate::db::get_pool;
use crate::users::users::db::UserDb;
use crate::auth::password_reset::db::PasswordResetDb;
use crate::auth::password_reset::forms::{ResetRequestForm, ResetConfirmForm};
use crate::email::send_plain_email;
use crate::validation::field_error;
use crate::errors::Error;
use crate::http::JsonResp;
use crate::http::Resp;

const RESET_TTL_MINUTES: i64 = 60;
const RESET_COOLDOWN_SEC: i64 = 60;

fn gen_token() -> String {
	let mut bytes = [0u8; 32];
	thread_rng().fill_bytes(&mut bytes);
	URL_SAFE_NO_PAD.encode(bytes)
}

fn hash_token(token: &str) -> String {
	let mut hasher = Sha256::new();
	hasher.update(token.as_bytes());
	format!("{:x}", hasher.finalize())
}

pub async fn request_reset(form: &ResetRequestForm) -> Resp {
	let pool = get_pool();
	let userdb = UserDb::new(pool.clone());
	let db = PasswordResetDb::new(pool.clone());

	let in_cooldown = db.is_in_cooldown(&form.email, RESET_COOLDOWN_SEC).await;
	let user = userdb.by_email(&form.email).await;
	if !in_cooldown {
		if let Some(user) = user {
			let token = gen_token();
			let token_hash = hash_token(&token);
			let expires_at = (Utc::now() + Duration::minutes(RESET_TTL_MINUTES)).to_rfc3339();
			let _ = db
				.insert(user.id, &form.email, &token_hash, &expires_at, None, None)
				.await;
			let link = format!("{}/auth/password-reset?token={}", form.base_url, token);
			let subject = "Восстановление пароля";
			let body = format!(
				"Чтобы установить новый пароль, перейдите по ссылке:\n\n{link}\n\nСсылка действует 60 минут."
			);
			let _ = send_plain_email(&form.email, subject, &body).await;
		}
	}

	JsonResp::ok("Если email зарегистрирован, ссылка будет отправлена.").to_http()
}

pub async fn confirm_reset(form: &ResetConfirmForm) -> Resp {
	let pool = get_pool();
	let userdb = UserDb::new(pool.clone());
	let db = PasswordResetDb::new(pool.clone());

	let token_hash = hash_token(&form.token);
	let entry = db.by_token_hash(&token_hash).await;
	let Some(entry) = entry else {
		let e = field_error("token", "bad_token", None);
		return JsonResp::err("Invalid token", &Error::Validation).content(&e).to_http();
	};

	if entry.used_at.is_some() {
		let e = field_error("token", "token_used", None);
		return JsonResp::err("Invalid token", &Error::Validation).content(&e).to_http();
	}

	let Ok(expires_at) = entry.expires_at.parse::<chrono::DateTime<Utc>>() else {
		let e = field_error("token", "bad_token", None);
		return JsonResp::err("Invalid token", &Error::Validation).content(&e).to_http();
	};
	if Utc::now() > expires_at {
		let e = field_error("token", "token_expired", None);
		return JsonResp::err("Invalid token", &Error::Validation).content(&e).to_http();
	}

	let updated = userdb.set_password(entry.user_id, &form.pwd).await;
	if !updated {
		return JsonResp::err("Update failed", &Error::Common).to_http();
	}
	let _ = db.mark_used(entry.id).await;
	JsonResp::ok("Пароль обновлён.").to_http()
}
