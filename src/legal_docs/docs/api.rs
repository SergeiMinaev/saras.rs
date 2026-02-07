use crate::serde_json::json;
use crate::http::{Request, Resp};
use crate::http;
use crate::http::JsonResp;
use crate::errors::Error;
use crate::request::RequestTools;
use crate::legal_docs::docs::db::DocDb;
use crate::legal_docs::docs::service;
use crate::legal_docs::docs::forms::make_doc_form;
use crate::db::get_pool;

pub async fn docs(req: Request) -> Resp {
    match req.method.as_str() {
        "get" => {
            if req.route.get("id").is_some() {
                return get_doc(req).await
            } else {
                return get_docs(req).await
            }
        },
        "post" | "put" | "delete" => {
            if req.is_su().await {
                match req.method.as_str() {
                    "post" => return create_doc(req).await,
                    "put" => return update_doc(req).await,
                    "delete" => return delete_doc(req).await,
                    _ => unreachable!(),
                }
            } else {
                return http::forbidden()
            }
        },
        _ => return http::not_found()
    }
}

pub async fn get_doc(req: Request) -> Resp {
    let id: i32 = req.route.get("id").unwrap().parse().unwrap();
    let pool = get_pool();
    let docdb = DocDb::new(pool.clone());
    match docdb.by_id(id).await {
        Some(doc) => {
            let r = json!(doc);
            JsonResp::ok("").content(&r).to_http()
        },
        None => http::not_found()
    }
}

pub async fn get_docs(_req: Request) -> Resp {
    let pool = get_pool();
    let docdb = DocDb::new(pool.clone());
    docdb.total_count().await;
    let r = docdb.page(0, 50).await;
    JsonResp::ok("").content(&r).to_http()
}

pub async fn create_doc(req: Request) -> Resp {
    match make_doc_form(&req) {
        Err(e) => JsonResp::err("Не удалось создать контентный блок.", &Error::Validation)
            .content(&e).to_http(),
        Ok(form) => {
            match service::create_doc(form).await {
                Err(_e) => JsonResp::err("Не удалось создать юр. документ.", &Error::Common)
                    .to_http(),
                Ok(doc) => JsonResp::ok("Юр. документ сохранён.").content(&doc).to_http()
            }
        }
    }
}

pub async fn update_doc(req: Request) -> Resp {
    match make_doc_form(&req) {
        Err(e) => JsonResp::err("Не удалось изменить юр. документ.", &Error::Validation)
            .content(&e).to_http(),
        Ok(form) => {
            let id: i32 = req.route.get("id").unwrap().parse().unwrap();
            match service::update_doc(id, form).await {
                Err(e) => JsonResp::err("Не удалось изменить юр. документ.", &e).to_http(),
                Ok(doc) => JsonResp::ok("Юр. документ изменён.").content(&doc).to_http()
            }
        }
    }
}

pub async fn delete_doc(req: Request) -> Resp {
    let id: i32 = req.route.get("id").unwrap().parse().unwrap();
    match service::delete_doc(id).await {
        false => JsonResp::err("Не удалось удалить юр. документ.", &Error::Common).to_http(),
        true => JsonResp::ok("Юр. документ удалён.").to_http(),
    }
}
