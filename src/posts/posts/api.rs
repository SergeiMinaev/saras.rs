use crate::serde_json::json;
use crate::http::{Request,Resp};
use crate::http;
use crate::http::JsonResp;
use crate::errors::Error;
use crate::request::RequestTools;
use crate::posts::posts::models;
use crate::posts::posts::db::PostDb;
use crate::posts::posts::service;
use crate::posts::posts::forms::{ CreatePostForm, UpdatePostForm  };
use crate::posts::posts::forms::PostForm;


pub async fn posts(req: Request) -> Resp {
	match req.method.as_str() {
	    "get" => {
	  		if req.route.get("id").is_some() {
				return get_post(req).await
			} else {
				return get_posts(req).await
			}
	    },
	    "post" | "put" | "delete" => {
			if req.is_su() {
				match req.method.as_str() {
					"post" => return create_post(req).await,
					"put" => return update_post(req).await,
					"delete" => return delete_post(req).await,
					_ => unreachable!(),
				}
			} else {
				return http::forbidden()
			}
	    },
		_ => return http::not_found()
	}
}


pub async fn get_post(req: Request) -> Resp {
	let id: i32 = req.route.get("id").unwrap().parse().unwrap();
	match PostDb::by_id(id) {
		Some(post) => {
			let r = json!(post);
			return JsonResp::ok("").content(&r).to_http()
		},
		None => return http::not_found()
	}
}


pub async fn get_posts(_req: Request) -> Resp {
	PostDb::total_count();
	let r = PostDb::page(0,20);
	return JsonResp::ok("").content(&r).to_http()
}


pub async fn create_post(req: Request) -> Resp {

	match CreatePostForm::validate(&req) {
		Err(e) => JsonResp::err("Не удалось создать статью.", &Error::Validation)
			.content(&e).to_http(),
		Ok(post_form) => {
			match service::create_post(post_form) {
				Err(e) => JsonResp::err("Не удалось создать статью.", &Error::Common)
					.content(&e).to_http(),
				Ok(post) => JsonResp::ok("Статья сохранена.").content(&post).to_http()
			}
		}
	}
}


pub async fn update_post(req: Request) -> Resp {
	match UpdatePostForm::validate(&req) {
        Err(e) => JsonResp::err("Не удалось изменить пользователя.", &Error::Validation).to_http(),
        Ok(post_form) => {
			let id: i32 = req.route.get("id").unwrap().parse().unwrap();
			match service::update_post(id, post_form).await {
				Err(e) => JsonResp::err("Не удалось изменить пользователя", &e).to_http(),
				Ok(post) => JsonResp::ok("Пользователь изменён.").content(&post).to_http()
			}
		}
	}
}


pub async fn delete_post(req: Request) -> Resp {
	let id: i32 = req.route.get("id").unwrap().parse().unwrap();
	match service::delete_post(id) {
		false => JsonResp::err("Не удалось удалить пользователя.", &Error::Common).to_http(),
		true => JsonResp::ok("Пользователь удалён.").to_http(),
	}
}
