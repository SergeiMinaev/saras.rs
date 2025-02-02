use serde::Deserialize;
use serde_json::json;
use crate::errors::Error;
use crate::http::{ Request,Resp, json_resp, del_session_resp };
use crate::http::JsonResp;
use crate::request::RequestTools;
use crate::db::get_pool;
use crate::users::users::db::UserDb;
use crate::auth::auth::db::AuthDb;


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

pub async fn login(req: Request) -> Resp {
    match serde_json::from_str::<LoginInput>(&req.body_string) {
        Err(_) => {
            return json_resp(401, r#"{"err": "bad_login_input"}"#.to_string())
        },
        Ok(u_) => {
            if u_.is_valid() == false {
                return json_resp(401, r#"{"err": "bad_login_input"}"#.to_string())
            }
			let pool = get_pool();
			let userdb = UserDb::new(pool.clone());
			let authdb = AuthDb::new(pool.clone());
            match userdb.by_email(&u_.email).await {
                None => {
                    return json_resp(401, r#"{"err": "user_not_found"}"#.to_string())
                },
                Some(u) => {
					let hash = userdb.hash(u.id).await;
                    if u.check_password(u_.pwd, hash) == false {
                        return json_resp(
                            401, r#"{"err": "bad_pwd"}"#.to_string())
                    } else {
                        if let Some(sess) = authdb.add_session(Some(u.id)).await {
                            let r = json!(u);
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
            let j = json!(&user);
            return JsonResp::ok("").content(&j).to_http()
        }
    }
}

pub async fn logout(_req: Request) -> Resp {
    del_session_resp()
}
