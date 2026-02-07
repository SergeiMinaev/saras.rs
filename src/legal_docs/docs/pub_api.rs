use crate::http::{Request, Resp};
use crate::http::JsonResp;
use crate::http;
use crate::legal_docs::docs::db::DocDb;
use crate::db::get_pool;
use crate::serde_json::json;

pub async fn doc_by_key(req: Request) -> Resp {
    let key = match req.route.get("key") {
        Some(v) => v,
        None => return http::not_found(),
    };
    let pool = get_pool();
    let docdb = DocDb::new(pool.clone());
    match docdb.active_by_key(key).await {
        Some(doc) => {
            let r = json!(doc);
            JsonResp::ok("").content(&r).to_http()
        },
        None => http::not_found(),
    }
}
