use serde::Deserialize;
use serde_json::json;
use crate::errors::Error;
use crate::http::{ Request,Resp, json_resp, del_session_resp };
use crate::http::JsonResp;
use crate::request::RequestTools;
use crate::db::get_pool;
use crate::users::users::db::UserDb;
use crate::auth::auth::db::AuthDb;
use crate::auth::auth::forms::make_regform;
use log::debug;
use crate::util::normalize_email;


#[derive(Debug, Deserialize)]
pub struct LoginInput {
    pub email: String,
    pub pwd: String,
}
impl LoginInput {
    pub fn is_valid(&self) -> bool {
        return self.email.len() < 255 && self.pwd.len() < 255
    }
}

#[derive(Debug, Deserialize)]
pub struct ChangePasswordInput {
    // У OAuth-аккаунтов пароля нет, текущий пароль не требуется — поэтому Option.
    pub current_pwd: Option<String>,
    pub new_pwd: String,
}


pub async fn login(req: Request) -> Resp {
    match serde_json::from_str::<LoginInput>(&req.body_string) {
        Err(_) => {
            return json_resp(401, r#"{"err": "bad_login_input"}"#.to_string())
        },
        Ok(u_) => {
            if u_.is_valid() == false {
                return json_resp(401, r#"{"err": "bad_login_input"}"#.to_string())
            }
			let email = normalize_email(&u_.email);
			let pool = get_pool();
			let userdb = UserDb::new(pool.clone());
			let authdb = AuthDb::new(pool.clone());
            match userdb.by_email(&email).await {
                None => {
                    return json_resp(401, r#"{"err": "user_not_found"}"#.to_string())
                },
                Some(u) => {
					let hash = userdb.hash(u.id).await;
					let has_password = !hash.is_empty();
                    if u.check_password(u_.pwd, hash) == false {
                        return json_resp(
                            401, r#"{"err": "bad_pwd"}"#.to_string())
                    } else {
                        if let Some(sess) = authdb.add_session(Some(u.id)).await {
                            let mut r = json!(u);
                            if let Some(obj) = r.as_object_mut() {
                                obj.insert("has_password".to_string(), json!(has_password));
                            }
                            return JsonResp::ok("").content(&r).session_id(sess.id).to_http()
                        } else {
                            return json_resp(
                                500, r#"{"err": "unknown_err"}"#.to_string())
                        }
                    }
                },
            }
        },
    }
}

pub async fn get_user(req: Request) -> Resp {
    match req.get_user().await {
		None => return JsonResp::err("unauthorized", &Error::Auth).code(401).to_http(),
        Some(user) => {
            let mut j = json!(&user);
            if let Some(obj) = j.as_object_mut() {
                obj.insert("has_password".to_string(), json!(!user.hash.is_empty()));
            }
            return JsonResp::ok("").content(&j).to_http()
        }
    }
}

pub async fn change_password(req: Request) -> Resp {
    let user = match req.get_user().await {
        None => return JsonResp::err("unauthorized", &Error::Auth).code(401).to_http(),
        Some(u) => u,
    };
    let input = match serde_json::from_str::<ChangePasswordInput>(&req.body_string) {
        Err(_) => return JsonResp::err("Некорректные данные.", &Error::Validation).to_http(),
        Ok(i) => i,
    };
    let pool = get_pool();
    let userdb = UserDb::new(pool.clone());
    let hash = userdb.hash(user.id).await;
    // Парольный аккаунт — проверяем текущий пароль. OAuth-аккаунт (пустой хеш)
    // задаёт пароль впервые, текущий не требуется.
    if !hash.is_empty() {
        let current = input.current_pwd.unwrap_or_default();
        if user.check_password(current, hash) == false {
            return JsonResp::err("Неверный текущий пароль.", &Error::Validation).to_http()
        }
    }
    match crate::users::users::service::set_password(user.id.try_into().unwrap(), &input.new_pwd).await {
        Ok(()) => JsonResp::ok("Пароль обновлён.").to_http(),
        Err(_) => JsonResp::err("Пароль не должен быть короче 12 символов.", &Error::Validation).to_http(),
    }
}

pub async fn logout(_req: Request) -> Resp {
    del_session_resp()
}
