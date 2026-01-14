use crate::http::{Request,Resp};
use crate::http;
use crate::http::JsonResp;
use crate::errors::Error;
use crate::users::profile::service;
use crate::users::profile::forms::SetNameForm;
use crate::validation::{field_error, parse_json_validation};
use crate::forms::image_field_form::ImageFieldForm;
use crate::users::avatars::service::{update_avatar, delete_avatar};
use crate::request::RequestTools;


pub async fn name(req: Request) -> Resp {
    match req.method.as_str() {
      "put" => return set_name(req).await,
      _ => return http::not_found()
    }
}

pub async fn avatar(req: Request) -> Resp {
    match req.method.as_str() {
      "put" => return set_avatar(req).await,
      _ => return http::not_found()
    }
}


pub async fn set_name(req: Request) -> Resp {
	match parse_json_validation::<SetNameForm>(&req.body_string) {
        Err(e) => JsonResp::err("Не удалось сохранить имя.", &Error::Validation)
			.content(&e)
			.to_http(),
        Ok(form) => {
			let user = req.get_user().await.unwrap();
			match service::set_name(user.id, &form.name).await {
				false => JsonResp::err("Не удалось сохранить имя.", &Error::Database).to_http(),
				true => JsonResp::ok("Имя сохранено.").to_http(),
			}
		},
	}
}

pub async fn set_avatar(req: Request) -> Resp {
	let user = req.get_user().await.unwrap();
	match ImageFieldForm::validate(&req) {
		Err(e) => JsonResp::err("Не удалось сохранить аватар.", &Error::Validation)
			.content(&e)
			.to_http(),
		Ok(form) => {
			if form.del.unwrap_or(false) {
				match delete_avatar(user.id.try_into().unwrap()).await {
					Ok(()) => JsonResp::ok("Аватар удалён.").to_http(),
					Err(e) => JsonResp::err("Не удалось удалить аватар.", &e).to_http(),
				}
			} else if form.data_base64.is_none() {
				JsonResp::err("Не удалось сохранить аватар.", &Error::Validation)
					.content(&field_error("data_base64", "required", "Поле `data_base64` должно быть заполнено."))
					.to_http()
			} else {
				match update_avatar(user.id.try_into().unwrap(), &form).await {
					Ok(()) => JsonResp::ok("Аватар сохранён.").to_http(),
					Err(e) => JsonResp::err("Не удалось сохранить аватар.", &e).to_http(),
				}
			}
		},
	}
}
