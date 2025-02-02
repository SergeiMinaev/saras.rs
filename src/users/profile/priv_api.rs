use crate::http::{Request,Resp};
use crate::http;
use crate::http::JsonResp;
use crate::errors::Error;
use crate::users::profile::service;
use crate::users::profile::forms::SetNameForm;
use crate::request::RequestTools;


pub async fn name(req: Request) -> Resp {
    match req.method.as_str() {
      "put" => return set_name(req).await,
      _ => return http::not_found()
    }
}


pub async fn set_name(req: Request) -> Resp {
	match serde_json::from_str::<SetNameForm>(&req.body_string) {
        Err(_e) => JsonResp::err("Не удалось сохранить имя.", &Error::Validation).to_http(),
        Ok(form) => {
			let user = req.get_user().await.unwrap();
			match service::set_name(user.id, &form.name).await {
				false => JsonResp::err("Не удалось сохранить имя.", &Error::Database).to_http(),
				true => JsonResp::ok("Имя сохранено.").to_http(),
			}
		},
	}
}
