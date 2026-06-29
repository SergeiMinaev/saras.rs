use serde_json::json;
use crate::http::{Request,Resp};
use crate::http;
use crate::http::JsonResp;
use crate::errors::Error;
use crate::users::users::db::UserDb;
use crate::users::users::service;
use crate::users::users::forms::{ CreateUserForm, UpdateUserForm };
//use sl10n::define_l10n;
use crate::db::get_pool;
use log::debug;
//use crate::storage::msgs::Msgs;


//define_l10n! {
//    hello => {
//		en: "Hello, {name}!",
//		ru: "Привет, {name}!"
//	},
//}

//pub static MSGS: Lazy<Msgs> = Lazy::new(|| Msgs::new());

//pub fn t(key: Msg) -> String {
//	MSGS.msg(key, "ru")
//}


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
	let pool = get_pool();
	let userdb = UserDb::new(pool.clone());
    match userdb.by_id(id).await {
		Some(user) => {
			let r = json!(user);
			return JsonResp::ok("").content(&r).to_http()
		},
		None => return http::not_found()
	}
}

pub async fn get_users(_req: Request) -> Resp {
	let pool = get_pool();
	let userdb = UserDb::new(pool.clone());
	let page = _req.query.get("page")
		.and_then(|v| v.parse::<u64>().ok())
		.filter(|v| *v > 0)
		.unwrap_or(1);
	let size = _req.query.get("size")
		.and_then(|v| v.parse::<u64>().ok())
		.filter(|v| *v > 0 && *v <= 200)
		.unwrap_or(20);
	let offset = ((page - 1) * size) as i32;
	let limit = size as i32;
	let (sort_by, sort_dir) = crate::admin::sort::from_req(&_req);
	let q = _req
		.query
		.get("q")
		.or_else(|| _req.query.get("name"))
		.map(|v| v.trim().to_string())
		.filter(|v| !v.is_empty());

	let (users, total) = if let Some(query) = q {
		(
			userdb.page_by_name(offset, limit, &query, sort_by.as_deref(), sort_dir.as_deref()).await,
			userdb.total_count_by_name(&query).await,
		)
	} else {
		(
			userdb.page(offset, limit, sort_by.as_deref(), sort_dir.as_deref()).await,
			userdb.total_count().await,
		)
	};

	let total_u64 = if total < 0 { 0 } else { total as u64 };
	let total_pages = if total_u64 == 0 {
		Some(1)
	} else {
		Some(((total_u64 + size - 1) / size).max(1))
	};
	let pagination = http::Pagination {
		page,
		per_page: size,
		total: Some(total_u64),
		total_pages,
		next_page: total_pages.and_then(|tp| if page < tp { Some(page + 1) } else { None }),
		prev_page: if page > 1 { Some(page - 1) } else { None },
	};

	let mut resp = JsonResp::ok("");
	resp.content(&users);
	resp.pagination(pagination);
	resp.to_http()
}

pub async fn create_user(req: Request) -> Resp {
	match CreateUserForm::validate(&req) {
        Err(e) => JsonResp::err("Не удалось создать пользователя.", &Error::Validation)
			.content(&e).to_http(),
        Ok(user_form) => {
			match service::create_user(user_form).await {
				Err(e) => JsonResp::err("Не удалось создать пользователя.", &e).to_http(),
				Ok(user) => JsonResp::ok("Пользователь сохранён.").content(&user).to_http()
			}
		}
	}
}

pub async fn update_user(req: Request) -> Resp {
	match UpdateUserForm::validate(&req) {
        Err(e) => {
			debug!("{e:?}");
			return JsonResp::err("Не удалось изменить пользователя.",&Error::Validation)
				.content(&e)
				.to_http()
		},
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
    match service::delete_user(id).await {
        Err(_e) => JsonResp::err("Не удалось удалить пользователя.", &Error::Common).to_http(),
        Ok(()) => JsonResp::ok("Пользователь удалён.").to_http(),
    }
}
