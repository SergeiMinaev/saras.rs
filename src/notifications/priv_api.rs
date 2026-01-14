use serde_json::json;
use crate::validation::parse_json_validation;
use crate::http::{Request,Resp};
use crate::http;
use crate::http::JsonResp;
use crate::errors::Error;
use crate::notifications::forms::{PaginationForm, MarkReadForm, MarkAllBeforeForm};
use crate::notifications::service;
use crate::request::RequestTools;

pub async fn notifications(req: Request) -> Resp {
	match req.method.as_str() {
		"get" => return list(req).await,
		_ => return http::not_found()
	}
}

pub async fn unread(req: Request) -> Resp {
	match req.method.as_str() {
		"get" => return unread_count(req).await,
		_ => return http::not_found()
	}
}

pub async fn read(req: Request) -> Resp {
	match req.method.as_str() {
		"post" => return mark_read(req).await,
		_ => return http::not_found()
	}
}

pub async fn read_before(req: Request) -> Resp {
	match req.method.as_str() {
		"post" => return mark_all_before(req).await,
		_ => return http::not_found()
	}
}

async fn list(req: Request) -> Resp {
	let user = match req.get_user().await {
		None => return JsonResp::err("unauthorized", &Error::Auth).code(401).to_http(),
		Some(u) => u
	};
	let form: PaginationForm = serde_json::from_str(&req.body_string).unwrap_or(PaginationForm{ offset: Some(0), size: Some(20) });
	let offset = form.offset.unwrap_or(0);
	let size = form.size.unwrap_or(20);
	let items = service::list(user.id as i32, offset, size).await;
	let j = json!(items);
	JsonResp::ok("").content(&j).to_http()
}

async fn unread_count(req: Request) -> Resp {
	let user = match req.get_user().await {
		None => return JsonResp::err("unauthorized", &Error::Auth).code(401).to_http(),
		Some(u) => u
	};
	let n = service::unread_count(user.id as i32).await;
	let j = json!({ "count": n });
	JsonResp::ok("").content(&j).to_http()
}

async fn mark_read(req: Request) -> Resp {
	let user = match req.get_user().await {
		None => return JsonResp::err("unauthorized", &Error::Auth).code(401).to_http(),
		Some(u) => u
	};
	let form: MarkReadForm = match parse_json_validation(&req.body_string) {
		Ok(f) => f,
		Err(e) => {
			return JsonResp::err("Validation", &Error::Validation)
				.content(&e)
				.to_http()
		}
	};
	let ok = service::mark_read(user.id as i32, form.ids).await;
	match ok {
		false => JsonResp::err("Database", &Error::Database).to_http(),
		true => JsonResp::ok("ok").to_http(),
	}
}

async fn mark_all_before(req: Request) -> Resp {
	let user = match req.get_user().await {
		None => return JsonResp::err("unauthorized", &Error::Auth).code(401).to_http(),
		Some(u) => u
	};
	let form: MarkAllBeforeForm = match parse_json_validation(&req.body_string) {
		Ok(f) => f,
		Err(e) => {
			return JsonResp::err("Validation", &Error::Validation)
				.content(&e)
				.to_http()
		}
	};
	let n = service::mark_all_before(user.id as i32, &form.ts).await;
	let j = json!({ "updated": n });
	JsonResp::ok("").content(&j).to_http()
}
