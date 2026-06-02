use isahc::{ AsyncReadResponseExt };
use serde_json::Value;
use crate::errors::Error;
use crate::conf::CONF;
use crate::auth::auth::service;
use crate::auth::auth::models::Session;


pub async fn get_email_by_code(code: &String) -> Result<String, Error> {
    let conf = CONF.read().await;
    let url = format!("{}&code={code}", conf.vk_auth_url);
    let resp = isahc::get_async(&url).await.unwrap().text().await.unwrap();
    eprintln!("[vk-oauth] ВРЕМЕННО code={code} vk_response={resp}");
    let v: Value = match serde_json::from_str(&resp) {
        Ok(v) => v,
        Err(e) => {
            eprintln!("[vk-oauth] ВРЕМЕННО не удалось распарсить ответ VK как JSON: {e}");
            return Err(Error::HTTP);
        }
    };
	if v["error"].is_null() == false || v["email"].is_string() == false {
		eprintln!(
			"[vk-oauth] ВРЕМЕННО отказ: error={:?} email_is_string={}",
			v.get("error"), v["email"].is_string()
		);
		return Err(Error::HTTP)
	}
	Ok(v["email"].as_str().unwrap().to_string())
}

pub async fn try_login_by_code(code: &String) -> Result<Session, Error> {
	let email = get_email_by_code(code).await?;
	if let Some(session) = service::login_by_email(&email).await {
		return Ok(session)
	} else {
		return Err(Error::Common)
	}
}
