use serde_json::json;
use crate::http::{Request,Resp};
use crate::http;
use crate::http::JsonResp;
use crate::errors::Error;
use crate::users::users::models;
use crate::users::users::db::UserDb;
use crate::users::users::service;
use crate::users::users::forms::{ CreateUserForm, UpdateUserForm };
use crate::users::users::forms::UserForm;
use once_cell::sync::Lazy;
use async_lock::RwLock;
use sl10n::define_l10n;


define_l10n! {
    hello => {
		en: "Hello, {name}!",
		ru: "Привет, {name}!"
	},
}

pub static MSGS: Lazy<Msgs> = Lazy::new(|| Msgs::new());

pub fn t(key: Msg) -> String {
	MSGS.msg(key, "ru")
}


pub async fn users(req: Request) -> Resp {
    match req.method.as_str() {
      "get" => {
        if req.route.get("id").is_some() {
          return get_user(req).await
        } else {
          return get_users(req).await
        }
      },
      "post" => return create_user(req).await,
      "put" => return update_user(req).await,
      "delete" => return delete_user(req).await,
      _ => return http::not_found()
    }
}

pub async fn get_user(req: Request) -> Resp {
    let id: i32 = req.route.get("id").unwrap().parse().unwrap();
    match UserDb::by_id(id) {
		Some(user) => {
			let r = json!(user);
			return JsonResp::ok("").content(&r).to_http()
		},
		None => return http::not_found()
	}
}

pub async fn get_users(_req: Request) -> Resp {
    UserDb::total_count();
    let r = UserDb::page(0,20);
    return JsonResp::ok("").content(&r).to_http()
}

pub async fn create_user(req: Request) -> Resp {
	match CreateUserForm::validate(&req) {
        Err(e) => JsonResp::err("Не удалось создать пользователя.", &Error::Validation)
			.content(&e).to_http(),
        Ok(user_form) => {
			match service::create_user(user_form) {
				Err(e) => JsonResp::err("Не удалось создать пользователя.", &Error::Common)
					.content(&e).to_http(),
				Ok(user) => JsonResp::ok("Пользователь сохранён.").content(&user).to_http()
			}
		}
	}
}

pub async fn update_user(req: Request) -> Resp {
	match UpdateUserForm::validate(&req) {
        Err(e) => JsonResp::err("Не удалось изменить пользователя.", &Error::Validation).to_http(),
        Ok(user_form) => {
			let id: i32 = req.route.get("id").unwrap().parse().unwrap();
			match service::update_user(id, user_form).await {
				Err(e) => JsonResp::err("Не удалось изменить пользователя", &e).to_http(),
				Ok(user) => JsonResp::ok("Пользователь изменён.").content(&user).to_http()
			}
		}
	}
}

pub async fn delete_user(req: Request) -> Resp {
    let id: i32 = req.route.get("id").unwrap().parse().unwrap();
    match models::User::delete(id) {
        false => JsonResp::err("Не удалось удалить пользователя.", &Error::Common).to_http(),
        true => JsonResp::ok("Пользователь удалён.").to_http(),
    }
}

