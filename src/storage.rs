use std::path::PathBuf;
use std::fs::File;
use std::io::prelude::*;
use isahc::http::status::StatusCode;
use isahc::prelude::*;
use isahc::Body;
use url::Url;
use serde_json::json;
use crate::memstore::MEMSTORE;
use chrono::{Duration};
use rand::{thread_rng, Rng};
use rand::distributions::Alphanumeric;
use crate::conf::CONF;
use crate::util::norm_path;

pub mod img_storage;
pub mod errors;
pub mod storage;
pub mod util;
pub mod msgs;


pub async fn get_base_url() -> String {
	let conf = CONF.read().await;
	let base_url = &conf.selectel.api_base_url;
	let proj_id = &conf.selectel.proj_id;
	let container_name = &conf.selectel.container_name;
	format!("{base_url}/{proj_id}/{container_name}")
}


pub async fn get_api_url(path: &PathBuf) -> String {
	let base_url = get_base_url().await;
	format!("{base_url}/{}", &format!("{}", path.display()))
}


pub async fn _get_token() -> String {
	let conf = CONF.read().await;
	let account_id = &conf.selectel.account_id;
	let proj_name = &conf.selectel.proj_name;
	let svc_user_name = &conf.selectel.svc_user_name;
	let svc_user_pwd = &conf.selectel.svc_user_pwd;
	let url = "https://cloud.api.selcloud.ru/identity/v3/auth/tokens";
	let body = json!({
		"auth": {
			"identity":{
				"methods":["password"],
				"password":{
					"user":{
						"name": svc_user_name,
						"domain":{"name": account_id},
						"password": svc_user_pwd,
					}
				}
			},
			"scope":{
				"project": {
					"name": proj_name,
					"domain": {"name": account_id}
				}
			}
		}
	});
	let resp = isahc::Request::builder()
		.method("POST")
		.uri(url)
		.header("Content-Type", "application/json")
		.body(Body::from(body.to_string()))
		.unwrap()
		.send()
		.unwrap();
	let token: String = resp.headers().get("X-Subject-Token").unwrap().to_str().unwrap().into();
	token
}

pub async fn get_token() -> String {
	let conf = CONF.read().await;
	let mut store = MEMSTORE.write().await;
	if let Some(token) = store.get("token") {
		return token.to_owned()
	}
	let token = _get_token().await;

	store.set(
		"token".into(),
		token.clone(),
		Some(Duration::try_seconds(conf.selectel.token_lifetime_sec).unwrap())
	);
	let token = store.get("token").unwrap();

	token.into()
}

pub async fn save_file(content: &Vec<u8>, path: &PathBuf) -> PathBuf {
	let mut path = path.clone();
	if is_file_exists(&path).await {
		path = uniquize_path(path);
	}
	if is_file_exists(&path).await {
		panic!("Path already exists: {}", path.display());
	}

	let token = get_token().await;
	let url = get_api_url(&path).await;

	let body = Body::from(content.clone());
	let resp = isahc::Request::builder()
		.method("PUT")
		.uri(url)
		.header("X-Auth-Token", token)
		.body(body)
		.unwrap()
		.send()
		.unwrap();
	if resp.status() != StatusCode::CREATED {
		println!("Error: {:?}", resp);
	}
	path
}

pub async fn is_file_exists(path: &PathBuf) -> bool {
	let token = get_token().await;
	let url = get_api_url(&path).await;
	let req = isahc::Request::builder()
		.method("GET")
		.uri(url.clone())
		.header("X-Auth-Token", token)
		.body(());
	let resp = req
		.unwrap()
		.send()
		.unwrap();
	resp.status() == StatusCode::OK
}


pub async fn delete_file(path: &PathBuf) {
	let token = get_token().await;
	let url = get_api_url(&path).await;
	let resp = isahc::Request::builder()
		.method("DELETE")
		.uri(url)
		.header("X-Auth-Token", token)
		.body(())
		.unwrap()
		.send()
		.unwrap();
	if resp.status() != StatusCode::NO_CONTENT {
		println!("Error: {:?}", resp);
	}
}


pub async fn ls_imgs(path: &String) -> Vec<String> {
	let token = get_token().await;
	let path = norm_path(format!("orig/{path}"));
	let path = path.strip_prefix("/").unwrap_or(&path);
	let mut base_url = Url::parse(&format!("{}?prefix={}/", get_base_url().await, path)).unwrap();
	base_url.query_pairs_mut().append_pair("delimiter", "/");
	let url = base_url.as_str();
	let mut resp = isahc::Request::builder()
		.method("GET")
		.uri(url)
		.header("X-Auth-Token", token.clone())
		.body(())
		.unwrap()
		.send()
		.unwrap();
	let data = resp.text().unwrap();
	data.split("\n")
		.filter( |s| *s != "" && (s.ends_with("jxl") || s.ends_with("/")) )
		.map( |s| {
			let s = s.strip_prefix("orig/").unwrap_or(s);
			if !s.ends_with("/") {
				let mut s = PathBuf::from(s);
				s.set_extension("");
				return s.into_os_string().into_string().unwrap()
			}
			s.to_string()
		})
		.collect()
}


pub async fn open_local_file(path: &PathBuf) -> Vec<u8> {
	let mut file = File::open(path).unwrap();
	let mut buffer: Vec<u8> = vec![];
	file.read_to_end(&mut buffer).unwrap();
	buffer
}


pub fn random_string(len: usize) -> String {
	thread_rng().sample_iter(&Alphanumeric).take(len).map(char::from).collect()
}


pub fn uniquize_path(mut path: PathBuf) -> PathBuf {	
	let stem = format!("{}", path.file_stem().unwrap().to_str().unwrap());
	let postfix = random_string(5);
	let ext: &str = path.extension().unwrap().to_str().unwrap();
	path.set_file_name(format!("{stem}_{postfix}.{ext}"));
	path
}
