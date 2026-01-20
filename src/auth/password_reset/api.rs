use crate::errors::Error;
use crate::http::{Request, Resp, JsonResp};
use crate::auth::password_reset::forms::{make_reset_request_form, make_reset_confirm_form};
use crate::auth::password_reset::service;
use serde_json::json;

pub async fn request(req: Request) -> Resp {
	match make_reset_request_form(&req) {
		Err(e) => {
			let e = json!(e);
			JsonResp::err("Ошибка валидации.", &Error::Validation).content(&e).to_http()
		}
		Ok(form) => service::request_reset(&form).await,
	}
}

pub async fn confirm(req: Request) -> Resp {
	match make_reset_confirm_form(&req) {
		Err(e) => {
			let e = json!(e);
			JsonResp::err("Ошибка валидации.", &Error::Validation).content(&e).to_http()
		}
		Ok(form) => service::confirm_reset(&form).await,
	}
}
