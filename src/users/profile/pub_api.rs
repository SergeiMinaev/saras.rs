use serde_json::json;
use crate::http::{Request,Resp};
use crate::http;
use crate::http::JsonResp;
use crate::errors::Error;
use crate::users::profile::service;
use crate::users::profile::forms::SetNameForm;
use crate::users::profile::db::ProfileDb;
use crate::request::RequestTools;
use crate::db::get_pool;


pub async fn profile(req: Request) -> Resp {
    match req.method.as_str() {
      "get" => return get_profile(req).await,
      _ => return http::not_found()
    }
}


pub async fn get_profile(req: Request) -> Resp {
	//let name: &str = req.route.get("name").unwrap();
    let id: i32 = req.route.get("id").unwrap().parse().unwrap();
	let pool = get_pool();
	let profiledb = ProfileDb::new(pool.clone());
	//match profiledb.by_name(name).await {
	match profiledb.by_id(id).await {
		Some(profile) => {
			let r = json!(profile);
			return JsonResp::ok("").content(&r).to_http()
		},
		None => return http::not_found()
	}
}
