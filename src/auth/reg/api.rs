
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
use crate::auth::reg::service;
use log::debug;



pub async fn reg(req: Request) -> Resp {
	match make_regform(&req) {
		Err(e) => {
			let e = json!(e);
			JsonResp::err("Ошибка валидации.", &Error::Validation).content(&e).to_http()
		},
		Ok(form) => {
			service::handle_reg(&form, &req).await
		}
	}
}
