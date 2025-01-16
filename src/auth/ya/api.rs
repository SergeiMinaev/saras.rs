use crate::http::{Request,Resp};
use crate::http;
use crate::http::JsonResp;
use crate::errors::Error;
use crate::auth::ya::service::try_login_by_code;
use crate::auth::ya::forms::{ AuthVkForm };
//use sl10n::define_l10n;
//use log::debug;


pub async fn index(req: Request) -> Resp {
	if req.method.as_str() != "post" { return http::not_found() }
	match AuthVkForm::validate(&req) {
		Err(e) => JsonResp::err("Форма заполнена некорректно.", &Error::Validation)
			.content(&e).to_http(),
		Ok(form) => {
			match try_login_by_code(&form.code).await {
				Err(_e) => JsonResp::err("Не удалось пройти аутентификацию.", &Error::Common)
					.to_http(),
				Ok(sess) => {
					return JsonResp::ok("Authenticated").session_id(sess.id).to_http()
				}
			}
		}
	}
}

