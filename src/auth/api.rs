use serde::Deserialize;
use serde_json::json;
use crate::http::{ Request,Resp, json_resp, del_session_resp };
use crate::http::JsonResp;
use crate::request::RequestTools;
use crate::users::users::models;


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
            match models::User::by_email(u_.email) {
                None => {
                    return json_resp(401, r#"{"err": "user_not_found"}"#.to_string())
                },
                Some(u) => {
                    if u.check_password(u_.pwd) == false {
                        return json_resp(
                            401, r#"{"err": "bad_pwd"}"#.to_string())
                    } else {
                        if let Some(sess) = u.add_session() {
                            //return session_resp(200, Some(sess.id));
                            let r = json!(u);
                            return JsonResp::ok("").content(r).session_id(sess.id).to_http()
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
    match req.get_user() {
		None => return JsonResp::err("unauthorized").code(401).to_http(),
        Some(user) => {
            let j = json!(&user);
            return JsonResp::ok("").content(j).to_http()
        }
    }
}

pub async fn logout(_req: Request) -> Resp {
    del_session_resp()
}
