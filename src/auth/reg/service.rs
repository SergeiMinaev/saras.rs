use log::debug;
use serde_json::json;
use crate::errors::Error;
use crate::http::{ Request, Resp, forbidden };
use crate::request::RequestTools;
use futures_lite::{ Future };
use crate::db::get_pool;
use crate::users::users::db::UserDb;
use crate::auth::auth::db::AuthDb;
use crate::users::users::forms::UserForm;
use crate::auth::auth::models::Session;
use crate::auth::auth::forms::RegForm;
use crate::memstore::MemStore;
use std::path::{Path, PathBuf};
use regex::Regex;
use futures_lite::{AsyncReadExt, StreamExt};
use once_cell::sync::Lazy;
use async_lock::RwLock;
use sha2::{Sha256, Digest};
use crate::conf::CONF;
use crate::legal_docs::consents;
use crate::http::JsonResp;
use crate::validation::field_error;
use chrono::{Duration, Utc, DateTime};
use rand::{Rng, thread_rng};
use serde::{Deserialize, Serialize};
use crate::users::avatars::service::update_default_avatar;
use crate::util::normalize_email;
use crate::email::send_plain_email;


static MEMSTORE: Lazy<RwLock<MemStore>> = Lazy::new(|| {
    RwLock::new(MemStore::new())
});

#[derive(Debug, Deserialize, Serialize, Clone)]
struct ConfirmationCode {
	code: String,
	errors: u32,
	created_at: DateTime<Utc>,
}

pub async fn is_ready_to_send_code(email: &str, sess_id: &str) -> bool {
	let mut store = MEMSTORE.write().await;
	let mut allow_by_email = false;
	let mut allow_by_sess = false;
	let cooldown = Duration::new(30, 0).unwrap();
	match store.get(email) {
		Some(code_str) => {
			let code: ConfirmationCode = serde_json::from_str(&code_str).unwrap();
			if code.created_at + cooldown < Utc::now() {
				allow_by_email = true;
			}
		},
		None => allow_by_email = true,
	}
	match store.get(sess_id) {
		Some(code_str) => {
			let code: ConfirmationCode = serde_json::from_str(&code_str).unwrap();
			if code.created_at + cooldown < Utc::now() {
				allow_by_sess = true;
			}
		},
		None => allow_by_sess = true,
	}
	return allow_by_email && allow_by_sess
}

pub async fn send_code(email: &str, sess_id: &str) -> String {
	let mut store = MEMSTORE.write().await;	
	let lifetime = Duration::new(300, 0).unwrap();
	let code = gen_code();
	let code_obj = ConfirmationCode {
		code: code.clone(),
		errors: 0,
		created_at: Utc::now(),
	};
	let code_str = serde_json::to_string(&code_obj).unwrap();
	store.set(email.to_string(), code_str.clone(), Some(lifetime));
	store.set(sess_id.to_string(), code_str, Some(lifetime));
	let conf = CONF.read().await;
	if conf.is_dev == false {
		send_code_email(email, &code.clone()).await;
	} else {
		println!("reg code: {code}");
	}
	code
}

pub async fn send_code_email(email: &str, code: &str) {
	let conf = crate::conf::CONF.read().await;
	let subject = format!("Код подтверждения {}", conf.site_name);
	drop(conf);
	let body = format!("Ваш код подтверждения: {code}");
	debug!("go send mail");
	match send_plain_email(email, &subject, &body).await {
		Ok(_) => println!("Письмо отправлено!"),
		Err(e) => eprintln!("Ошибка: {e}"),
	}
}

pub async fn is_code_active(email: &str) -> bool {
	let mut store = MEMSTORE.write().await;
	if let Some(code) = store.get(email) {
		debug!("check active: {code:?}");
		let mut code: ConfirmationCode = serde_json::from_str(&code).unwrap();
		return code.errors < 3
	}
	false
}

pub async fn check_code(email: &str, sess_id: &str, code: &str) -> bool {
	let mut store = MEMSTORE.write().await;
	match store.get(email) {
		Some(correct) => {
			let mut correct: ConfirmationCode = serde_json::from_str(&correct).unwrap();
			//debug!("{correct:?}");
			if correct.errors < 3 && correct.code == code {
				return true
			} else {
				correct.errors += 1;
				let code_str = serde_json::to_string(&correct).unwrap();
				let new_lifetime = Duration::new(180, 0).unwrap();
				store.set(email.to_string(), code_str.clone(), Some(new_lifetime));
				store.set(sess_id.to_string(), email.to_string(), Some(new_lifetime));
				return false
			}
		},
		None => {
			debug!("Code not exists");
			return false
		}
	}
}

pub async fn drop_code(email: &str, sess_id: &str) {
	let mut store = MEMSTORE.write().await;
	store.del(email);
	store.del(sess_id);
}


pub async fn handle_reg(form: &RegForm, req: &Request) -> Resp {
	let sess_id = &req.session_id;
	//debug!("{form:?}");
	let email = normalize_email(&form.email);
	if form.code.is_some() == false {
		if is_ready_to_send_code(&email, sess_id).await {
			let code = send_code(&email, sess_id).await;
			debug!("Code sent: {code}");
			JsonResp::ok("Введите код подтверждения. Он был отправлен на указанный email.").to_http()
		} else {
			let e = json!({"code": "cooldown"});
			JsonResp::ok("Подождите").content(&e).to_http()
		}
	} else {
		println!("Check code");
		let code = form.code.clone().unwrap();
		if is_code_active(&email).await == false {
			return JsonResp::err("Запросите новый код подтверждения.", &Error::Validation)
				.content(&field_error(
					"code",
					"code_inactive",
					"Запросите новый код подтверждения.",
				))
				.to_http()
		}
		let is_correct = check_code(&email, sess_id, &code).await;
		if is_correct {
			finish_reg(form, req, &email).await
		} else {
			JsonResp::err("Неправильный код подтверждения.", &Error::Validation)
				.content(&field_error(
					"code",
					"bad_code",
					"Неправильный код подтверждения.",
				))
				.to_http()
		}
	}
}

pub async fn finish_reg(form: &RegForm, req: &Request, email: &str) -> Resp {
	let userform = UserForm {
		name: Some(form.name.clone()),
		email: email.to_string(),
		pwd: Some(form.pwd.clone()),
		is_superuser: Some(false),
		avatar: None,
	};
	let pool = get_pool();
	let userdb = UserDb::new(pool);
	let user_id = match userdb.create(userform).await {
		Some(id) => id,
		None => {
			return JsonResp::err("Пользователь с таким email уже зарегистрирован.", &Error::Validation)
				.content(&field_error(
					"email",
					"email_exists",
					"Пользователь с таким email уже зарегистрирован.",
				))
				.to_http()
		}
	};
	smol::spawn(async move {
		let _ = update_default_avatar(user_id).await;
	}).detach();
	let consent_key = {
		let conf = CONF.read().await;
		conf.legal_docs.consent_key.clone()
	};
	let consent_version = {
		let conf = CONF.read().await;
		conf.legal_docs.consent_version.clone()
	};
	let ip = req.headers.get("x-real-ip")
		.cloned()
		.or_else(|| req.headers.get("x-forwarded-for")
			.and_then(|v| v.split(',').next().map(|s| s.trim().to_string()))
		)
		.filter(|v| !v.is_empty());
	let user_agent = req.headers.get("user-agent")
		.cloned()
		.filter(|v| !v.is_empty());
	let _ = consents::create_consent(
		user_id as i32,
		&consent_key,
		&consent_version,
		"web",
		ip,
		user_agent,
	).await;
	drop_code(email, &req.session_id).await;
	let j = json!({"reg_complete": true});
	JsonResp::ok("Регистрация завершена.").content(&j).to_http()
}

pub fn gen_code() -> String {
    let mut rng = thread_rng();
    (0..6).map(|_| rng.gen_range(0..=9).to_string()).collect()
}
