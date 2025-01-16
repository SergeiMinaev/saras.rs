use isahc::{ AsyncReadResponseExt, Request, RequestExt };
use serde_json::Value;
use log::debug;
use crate::errors::Error;
use crate::conf::CONF;
use crate::auth::auth::service;
use crate::auth::auth::models::Session;
use crate::util::encode_base64;


pub async fn get_email_by_code(code: &String) -> Result<String, Error> {
    let conf = CONF.read().await;
	let creds = encode_base64(&format!("{}:{}", conf.ya_auth_client_id, conf.ya_auth_client_secret));
	debug!("creds: {creds}");
	let body = format!("grant_type=authorization_code&code={code}");
	debug!("body: {body}");
	let resp = Request::post(&conf.ya_auth_url)
		.header("Authorization", format!("Basic {}", creds))
		.header("Content-Type", "application/x-www-form-urlencoded")
		.body(body)
		.unwrap()
		.send_async()
		.await
		.unwrap()
		.text()
		.await
		.unwrap();
    let v: Value = serde_json::from_str(&resp).unwrap();
	debug!("{v}");
	if v["access_token"].is_null() {
		debug!("{v}");
		return Err(Error::HTTP)
	}
	let token = v["access_token"].as_str().unwrap();
	get_email_by_token(token).await
}

pub async fn get_email_by_token(token: &str) -> Result<String, Error> {
	let url = "https://login.yandex.ru/info?format=json";
	let resp = Request::get(url)
		.header("Authorization", format!("OAuth {token}"))
		.body(())
		.unwrap().send_async().await.unwrap().text().await.unwrap();
    let user_info: Value = serde_json::from_str(&resp).unwrap();
	if user_info["default_email"].is_null() {
		debug!("info: {user_info}");
		return Err(Error::HTTP)
	}
	Ok(user_info["default_email"].as_str().unwrap().to_string())
}

pub async fn try_login_by_code(code: &String) -> Result<Session, Error> {
	let email = get_email_by_code(code).await?;
	if let Some(session) = service::login_by_email(&email).await {
		return Ok(session)
	} else {
		return Err(Error::Common)
	}
}
