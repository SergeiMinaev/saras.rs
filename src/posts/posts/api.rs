use crate::serde_json::json;
use crate::http::{Request,Resp};
use crate::http;
use crate::http::JsonResp;
use crate::errors::Error;
use crate::request::RequestTools;
use crate::posts::posts::db::PostDb;
use crate::posts::posts::service;
use crate::posts::posts::forms::{ CreatePostForm, UpdatePostForm  };
use crate::db::get_pool;


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
			if req.is_su().await {
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
	let pool = get_pool();
	let postdb = PostDb::new(pool.clone());
	match postdb.by_id(id).await {
		Some(post) => {
			let r = json!(post);
			return JsonResp::ok("").content(&r).to_http()
		},
		None => return http::not_found()
	}
}


pub async fn get_posts(_req: Request) -> Resp {
	let pool = get_pool();
	let postdb = PostDb::new(pool.clone());
	postdb.total_count().await;
	let (sort_by, sort_dir) = crate::admin::sort::from_req(&_req);
	let r = postdb.page(0, 20, sort_by.as_deref(), sort_dir.as_deref()).await;
	return JsonResp::ok("").content(&r).to_http()
}


pub async fn create_post(req: Request) -> Resp {

	match CreatePostForm::validate(&req) {
		Err(e) => JsonResp::err("Не удалось создать статью.", &Error::Validation)
			.content(&e).to_http(),
		Ok(post_form) => {
			match service::create_post(post_form).await {
				Err(e) => JsonResp::err("Не удалось создать статью.", &Error::Common)
					.content(&e).to_http(),
				Ok(post) => JsonResp::ok("Статья сохранена.").content(&post).to_http()
			}
		}
	}
}


pub async fn update_post(req: Request) -> Resp {
	match UpdatePostForm::validate(&req) {
        Err(e) => JsonResp::err("Не удалось изменить пост.", &Error::Validation)
			.content(&e).to_http(),
        Ok(post_form) => {
			let id: i32 = req.route.get("id").unwrap().parse().unwrap();
			match service::update_post(id, post_form).await {
				Err(e) => JsonResp::err("Не удалось изменить пост.", &e).to_http(),
				Ok(post) => JsonResp::ok("Пост изменён.").content(&post).to_http()
			}
		}
	}
}


pub async fn delete_post(req: Request) -> Resp {
	let id: i32 = req.route.get("id").unwrap().parse().unwrap();
	match service::delete_post(id).await {
		false => JsonResp::err("Не удалось удалить пост.", &Error::Common).to_http(),
		true => JsonResp::ok("Пост удалён.").to_http(),
	}
}
