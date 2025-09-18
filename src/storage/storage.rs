use std::path::{Path, PathBuf};
use chrono::{Duration};
use isahc::http::status::StatusCode;
use isahc::prelude::*;
use isahc::Body;
use std::io::Cursor;
use brotli::Decompressor;
//use log::debug;
use serde_json::json;
use rand::{thread_rng, Rng};
use rand::distributions::Alphanumeric;
use crate::errors::Error;
use crate::conf::CONF;
use crate::memstore::MEMSTORE;
//use crate::storage::msgs;
use crate::util::slugify;


const TOKEN_KEY: &str = "storage_token";


pub async fn get_base_url() -> String {
	let conf = CONF.read().await;
	let base_url = &conf.selectel.api_base_url;
	let proj_id = &conf.selectel.proj_id;
	let container_name = &conf.selectel.container_name;
	format!("{base_url}/{proj_id}/{container_name}")
}


pub async fn get_api_url<P: AsRef<Path>>(path: P) -> String {
	let base_url = get_base_url().await;
	format!("{base_url}/{}", &format!("{}", path.as_ref().display()))
}

pub fn random_string(len: usize) -> String {
	thread_rng().sample_iter(&Alphanumeric).take(len).map(char::from).collect()
}


pub fn randomize_path(mut path: PathBuf) -> PathBuf {
	let stem = path.file_stem().unwrap_or_else(|| path.as_os_str());
	let stem = stem.to_string_lossy();
	let postfix = random_string(5);
	let new_file_name = if let Some(ext) = path.extension() {
		let ext = ext.to_string_lossy();
		format!("{stem}_{postfix}.{ext}")
	} else {
		format!("{stem}_{postfix}")
	};
	path.set_file_name(new_file_name);
	path
}

pub struct Storage {
}

impl Storage {
	pub fn new() -> Self {
		Self {}
	}
}
impl Storage {
	pub async fn exists<P: AsRef<Path>>(&self, path: P) -> Result<bool, Error>{
		let token = self.get_token().await;
		let url = get_api_url(&path).await;
		let req = isahc::Request::builder()
			.method("GET")
			.uri(url.clone())
			.header("X-Auth-Token", token)
			.body(());
		let resp = req.unwrap().send().map_err(|_| Error::Storage)?;
		Ok(resp.status() == StatusCode::OK)
	}
	pub async fn open(&self, path: &PathBuf) -> Result<Vec<u8>, Error> {
		let url = get_api_url(path).await;
		let req = isahc::Request::builder()
			.method("GET")
			.uri(url)
			.header("X-Auth-Token", self.get_token().await)
			.body(());
		let mut resp = req.unwrap().send().map_err(|_| Error::Storage)?;
		if resp.status() != StatusCode::OK {
			return Err(Error::Storage)
		}
		let bytes = resp.bytes().map_err(|_| Error::Storage)?;
		let is_br = resp.headers().get("Content-Encoding")
			.and_then(|v| v.to_str().ok())
			.map(|s| s.eq_ignore_ascii_case("br"))
			.unwrap_or(false);
		if is_br {
			let mut out: Vec<u8> = Vec::new();
			let mut cursor = Cursor::new(bytes);
			let mut dec = Decompressor::new(&mut cursor, 4096);
			std::io::copy(&mut dec, &mut out).map_err(|_| Error::Storage)?;
			Ok(out)
		} else {
			Ok(bytes)
		}
	}
	pub async fn ls(&self, path: &PathBuf) -> Vec<String> {
		let url = format!(
			"{}?delimiter=/&prefix={}/",
			get_base_url().await, path.display()
		);
		//debug!("url: {url}");
		let mut resp = isahc::Request::builder()
			.method("GET")
			.uri(url.clone())
			.header("X-Auth-Token", self.get_token().await)
			.body(())
			.unwrap()
			.send()
			.unwrap();
		let data = resp.text().unwrap();
		data.split("\n")
			.filter( |s| !s.is_empty() && (s.contains(".") || s.ends_with("/")) )
			.map( |s| s.strip_prefix("orig/").unwrap_or(s).to_string())
			.collect()
	}
	pub async fn delete<P: AsRef<Path>>(&self, path: P) -> Result<(), Error>{
		//debug!("Storage delete path {}", path.as_ref().display());
		let token = self.get_token().await;
		let url = get_api_url(&path).await;
		//debug!("Storage delete url {}", url);
		let resp = isahc::Request::builder()
			.method("DELETE")
			.uri(url)
			.header("X-Auth-Token", token)
			.body(())
			.unwrap()
			.send().
			map_err(|_| Error::Storage)?;
		//debug!("storage resp: {resp:?}");
		//debug!("storage status: {:?}", resp.status());
		match resp.status() {
			StatusCode::NO_CONTENT => return Ok(()),
			_ => return Err(Error::Storage),
		}
	}
	pub async fn save(&self, data: Vec<u8>, path: &PathBuf) -> Result<PathBuf, Error> {
		let is_fixed_path = false;
		let is_force_overwrite = false;
		self._save(data, path, is_fixed_path, is_force_overwrite, false).await
	}

	/// Save brotli-compressed `data` making sure the object is uploaded
	/// with the `Content-Encoding: br` header.
	///
	/// Pass the desired final object `path` *without* the `.br` suffix:
	/// browsers will download and transparently decompress it.
	pub async fn save_br(&self, data: Vec<u8>, path: &PathBuf) -> Result<PathBuf, Error> {
		let compressed = crate::storage::util::compress_br(&data);
		let is_fixed_path = false;
		let is_force_overwrite = false;
		self._save(compressed, path, is_fixed_path, is_force_overwrite, true).await
	}
	pub async fn save_fixed(&self, data: Vec<u8>, path: &PathBuf) -> Result<(), Error> {
		let is_fixed_path = true;
		let is_force_overwrite = false;
		self._save(data, path, is_fixed_path, is_force_overwrite, false).await.map(|_| ())
	}
	pub async fn save_force_overwrite(&self, data: Vec<u8>, path: &PathBuf) -> Result<(), Error> {
		let is_fixed_path = true;
		let is_force_overwrite = true;
		self._save(data, path, is_fixed_path, is_force_overwrite, false).await.map(|_| ())
	}
	pub async fn _save(&self, data: Vec<u8>, path: &PathBuf,
		fixed_path: bool, is_force_overwrite: bool, is_br: bool
	) -> Result<PathBuf, Error> {
		let mut path = path.clone();
		if fixed_path {
			if self.exists(&path).await? {
				if !is_force_overwrite {
					return Err(Error::Common)
				}
			}
		} else {
			path = self.get_unique_path(&path).await?;
		}

		let url = get_api_url(&path).await;
		//debug!("Storage save url{}", url);

		// add Content-Encoding: br when uploading pre-compressed *.br objects
		let is_br = is_br || path
			.extension()
			.and_then(|e| e.to_str())
			.map(|s| s.eq_ignore_ascii_case("br"))
			.unwrap_or(false);

		let mut req_builder = isahc::Request::builder()
			.method("PUT")
			.uri(url)
			.header("X-Auth-Token", self.get_token().await);

		if is_br {
			req_builder = req_builder.header("Content-Encoding", "br");
		}

		let resp = req_builder
			.body(Body::from(data))
			.unwrap()
			.send()
			.unwrap();
		if resp.status() != StatusCode::CREATED {
			//debug!("API resp: {resp:?}");
			return Err(Error::Storage)
		}
		Ok(path)
	}
	pub async fn get_token(&self) -> String {
		let conf = CONF.read().await;
		let mut store = MEMSTORE.write().await;
		if let Some(token) = store.get(TOKEN_KEY) {
			return token.to_owned()
		} 
		let token = self.get_new_token().await;

		store.set(
			TOKEN_KEY.into(),
			token.clone(),
			Some(Duration::try_seconds(conf.selectel.token_lifetime_sec).unwrap())
		);
		let token = store.get(TOKEN_KEY).unwrap();

		token.into()
	}
	pub async fn get_new_token(&self) -> String {
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
	pub async fn get_unique_path(&self, path: &PathBuf) -> Result<PathBuf, Error> {
		let path = PathBuf::from(slugify(path.clone()));
		let mut new_path = path.clone();
		while self.exists(&new_path).await? {
			new_path = randomize_path(path.clone());
		}
		Ok(new_path.into())
	}
}
