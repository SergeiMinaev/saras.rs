use serde_json::json;
use crate::http::{Request,Resp};
use crate::http;
use crate::http::JsonResp;
use crate::users::users::models;
use crate::users::users::input;
use crate::users::users::view_models;


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
    let user = view_models::UserView::by_id(id);
    if user.is_some() {
      let r = json!(user.unwrap());
      return JsonResp::ok("").content(r).to_http()
    } else {
      return http::not_found()
    }
}

pub async fn get_users(_req: Request) -> Resp {
    view_models::UserView::total_count();
    let r = json!(view_models::UserView::page(0,20));
    return JsonResp::ok("").content(r).to_http()
}

pub async fn create_user(req: Request) -> Resp {
    match input::UserInput::create(&req).await {
        Err(e) => JsonResp::err("Не удалось создать пользователя.").content(e).to_http(),
        Ok(user) => {
          JsonResp::ok("Пользователь сохранён.").content(json!(user)).to_http()
        }
    }
}

pub async fn update_user(req: Request) -> Resp {
    match input::UserInput::update(&req).await {
        Err(e) => JsonResp::err("Не удалось сохранить пользователя.").content(e).to_http(),
        Ok(user) => {
          JsonResp::ok("Пользователь сохранён.").content(json!(user)).to_http()
        }
    }
}

pub async fn delete_user(req: Request) -> Resp {
    let id: i32 = req.route.get("id").unwrap().parse().unwrap();
    match models::User::delete(id) {
        false => JsonResp::err("Не удалось удалить пользователя.").to_http(),
        true => JsonResp::ok("Пользователь удалён.").to_http(),
    }
}

