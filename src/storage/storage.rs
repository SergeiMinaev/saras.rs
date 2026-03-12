use std::path::{Path, PathBuf};
use chrono::{Duration};
use hmac::{Hmac, Mac};
use isahc::http::status::StatusCode;
use isahc::prelude::*;
use isahc::Body;
use isahc::AsyncBody;
use std::io::Cursor;
use std::io::Read;
use std::io::Write;
use std::time::{Duration as StdDuration, Instant};
use brotli::Decompressor;
use async_std::task;
//use log::debug;
use serde_json::json;
use rand::{thread_rng, Rng};
use rand::distributions::Alphanumeric;
use crate::errors::Error;
use crate::conf::CONF;
use crate::memstore::MEMSTORE;
//use crate::storage::msgs;
use crate::util::slugify;
use futures_lite::io::AsyncReadExt;
use sha2::{Digest, Sha256};
use url::Url;
use url::form_urlencoded::byte_serialize;
use base64::Engine;


const TOKEN_KEY: &str = "storage_token";
const STREAM_PROGRESS_STEP_BYTES: u64 = 100 * 1024 * 1024;
const STREAM_PROGRESS_FIRST_BYTES: u64 = 1 * 1024 * 1024;
const STREAM_PROGRESS_TIME_SECS: u64 = 5;


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
	use_map: bool,
	container_override: Option<String>,
	api_base_override: Option<String>,
}

#[derive(Debug, Clone)]
pub struct ContainerCors {
	pub allow_origin: String,
	pub allow_methods: String,
	pub allow_headers: String,
	pub expose_headers: String,
	pub max_age: u32,
}

#[derive(Debug, Clone)]
pub struct ContainerCorsState {
	pub allow_origin: Option<String>,
	pub allow_methods: Option<String>,
	pub allow_headers: Option<String>,
	pub expose_headers: Option<String>,
	pub max_age: Option<u32>,
}

impl Storage {
	pub fn new() -> Self {
		Self {
			use_map: false,
			container_override: None,
			api_base_override: None,
		}
	}
	pub fn with_map() -> Self {
		Self {
			use_map: true,
			container_override: None,
			api_base_override: None,
		}
	}
	pub fn with_container(container: impl Into<String>) -> Self {
		let container = container.into().trim().to_string();
		assert!(
			!container.is_empty(),
			"[saras] Storage::with_container: empty container name"
		);
		Self {
			use_map: false,
			container_override: Some(container),
			api_base_override: None,
		}
	}
	pub fn with_container_and_base_url(
		container: impl Into<String>,
		api_base_url: impl Into<String>,
	) -> Self {
		let container = container.into().trim().to_string();
		let api_base_url = api_base_url.into().trim().to_string();
		assert!(
			!container.is_empty(),
			"[saras] Storage::with_container_and_base_url: empty container name"
		);
		assert!(
			!api_base_url.is_empty(),
			"[saras] Storage::with_container_and_base_url: empty api base url"
		);
		Self {
			use_map: false,
			container_override: Some(container),
			api_base_override: Some(api_base_url),
		}
	}
}
impl Storage {
	async fn base_url_for(&self) -> String {
		let conf = CONF.read().await;
		let base = if let Some(b) = self.api_base_override.as_ref() {
			b
		} else if self.use_map {
			&conf.selectel.map_api_base_url
		} else {
			&conf.selectel.api_base_url
		};
		let proj_id = &conf.selectel.proj_id;
		let container = if let Some(c) = self.container_override.as_ref() {
			c
		} else if self.use_map {
			&conf.selectel.map_container_name
		} else {
			&conf.selectel.container_name
		};
		format!("{base}/{proj_id}/{container}")
	}

	async fn api_url_for(&self, path: &Path) -> String {
		format!("{}/{}", self.base_url_for().await, path.display())
	}

	pub async fn exists<P: AsRef<Path>>(&self, path: P) -> Result<bool, Error>{
		let token = self.get_token().await;
		let url = self.api_url_for(path.as_ref()).await;
		let req = isahc::Request::builder()
			.method("HEAD")
			.uri(url.clone())
			.header("X-Auth-Token", token)
			.body(())
			.map_err(|_| Error::Storage)?;
		let mut resp = req.send_async().await.map_err(|e| {
			eprintln!(
				"[saras][storage] exists failed path={} err={:?}",
				path.as_ref().display(),
				e
			);
			Error::Storage
		})?;
		Ok(resp.status() == StatusCode::OK)
	}

	pub async fn stat_content_length<P: AsRef<Path>>(&self, path: P) -> Result<Option<u64>, Error>{
		let token = self.get_token().await;
		let url = self.api_url_for(path.as_ref()).await;
		let req = isahc::Request::builder()
			.method("HEAD")
			.uri(url.clone())
			.header("X-Auth-Token", token)
			.body(())
			.map_err(|_| Error::Storage)?;
		let resp = req.send_async().await.map_err(|e| {
			eprintln!(
				"[saras][storage] stat_content_length failed path={} err={:?}",
				path.as_ref().display(),
				e
			);
			Error::Storage
		})?;
		if resp.status() == StatusCode::NOT_FOUND {
			return Ok(None);
		}
		if resp.status() != StatusCode::OK {
			return Err(Error::Storage);
		}
		let len = resp
			.headers()
			.get("Content-Length")
			.or_else(|| resp.headers().get("content-length"))
			.and_then(|v| v.to_str().ok())
			.and_then(|s| s.parse::<u64>().ok());
		Ok(len)
	}
	pub async fn open(&self, path: &PathBuf) -> Result<Vec<u8>, Error> {
		// If `path` is an absolute URL (starts with http:// or https://),
		// use it directly; otherwise build the API URL based on the configured base.
		let url = match path.to_str() {
			Some(s) if s.starts_with("http://") || s.starts_with("https://") => s.to_string(),
			_ => self.api_url_for(path.as_path()).await,
		};

		// Build an async client with automatic decompression turned OFF so isahc
		// does not try to handle "br" itself (we will decode manually).
		// NOTE: building a new HttpClient on every call is wasteful; consider
		// reusing a single client instance stored on `Storage`.
		let client = isahc::HttpClient::builder()
			.automatic_decompression(false)
			.build()
			.map_err(|e| {
				eprintln!(
					"[saras][storage] open client build failed path={} err={:?}",
					path.display(),
					e
				);
				Error::Storage
			})?;

		let req = isahc::Request::builder()
			.method("GET")
			.uri(url.clone())
			.header("X-Auth-Token", self.get_token().await)
			.body(())
			.map_err(|e| {
				eprintln!(
					"[saras][storage] open request build failed path={} url={} err={:?}",
					path.display(),
					url,
					e
				);
				Error::Storage
			})?;

		let mut resp = client.send_async(req).await.map_err(|e| {
			eprintln!(
				"[saras][storage] open send failed path={} url={} err={:?}",
				path.display(),
				url,
				e
			);
			Error::Storage
		})?;
		if resp.status() != StatusCode::OK {
			// 404 is an expected "miss" in many call sites; avoid log spam.
			if resp.status() != StatusCode::NOT_FOUND {
				eprintln!(
					"[saras][storage] open non-200 path={} url={} status={}",
					path.display(),
					url,
					resp.status()
				);
			}
			return Err(Error::Storage)
		}

		// Read Content-Encoding header before consuming the response body,
		// because `into_body()` takes ownership of `resp`.
		let is_br_header = resp.headers().get("Content-Encoding")
			.and_then(|v| v.to_str().ok())
			.map(|s| s.eq_ignore_ascii_case("br"))
			.unwrap_or(false);

		// Read the async response body into a Vec<u8>.
		let mut body = resp.into_body();
		let mut bytes: Vec<u8> = Vec::new();
		futures_lite::io::AsyncReadExt::read_to_end(&mut body, &mut bytes)
			.await
			.map_err(|e| {
				eprintln!(
					"[saras][storage] open read failed path={} url={} err={:?}",
					path.display(),
					url,
					e
				);
				Error::Storage
			})?;

		let is_br = is_br_header
			|| path.extension()
				.and_then(|e| e.to_str())
				.map(|s| s.eq_ignore_ascii_case("br"))
				.unwrap_or(false);

		if is_br {
			let mut out: Vec<u8> = Vec::new();
			let mut cursor = Cursor::new(bytes);
			let mut dec = Decompressor::new(&mut cursor, 4096);
			std::io::copy(&mut dec, &mut out).map_err(|e| {
				eprintln!(
					"[saras][storage] open brotli decode failed path={} url={} err={:?}",
					path.display(),
					url,
					e
				);
				Error::Storage
			})?;
			Ok(out)
		} else {
			Ok(bytes)
		}
	}
	pub async fn ls(&self, path: &PathBuf) -> Vec<String> {
		let raw_prefix = path.display().to_string();
		let prefix = raw_prefix.trim_matches('/');
		let url = if prefix.is_empty() {
			format!("{}?delimiter=/", self.base_url_for().await)
		} else {
			format!("{}?delimiter=/&prefix={prefix}/", self.base_url_for().await)
		};
		//debug!("url: {url}");
		let mut resp = isahc::Request::builder()
			.method("GET")
			.uri(url.clone())
			.header("X-Auth-Token", self.get_token().await)
			.body(())
			.unwrap()
			.send_async().await
			.unwrap();

		// Read body into bytes and convert to UTF-8 string (preserve the previous unwrap behavior).
		let mut body = resp.into_body();
		let mut bytes: Vec<u8> = Vec::new();
		futures_lite::io::AsyncReadExt::read_to_end(&mut body, &mut bytes).await.unwrap();
		let data = String::from_utf8(bytes).unwrap();

		data.split("\n")
			.map(|s| s.trim())
			.filter(|s| !s.is_empty() && (s.contains(".") || s.ends_with("/")))
			.map(|s| s.strip_prefix("orig/").unwrap_or(s))
			.filter(|s| !s.is_empty())
			.map(|s| s.to_string())
			.collect()
	}
	pub async fn delete<P: AsRef<Path>>(&self, path: P) -> Result<(), Error>{
		//debug!("Storage delete path {}", path.as_ref().display());
		let token = self.get_token().await;
		let url = self.api_url_for(path.as_ref()).await;
		//debug!("Storage delete url {}", url);
		let mut resp = isahc::Request::builder()
			.method("DELETE")
			.uri(url)
			.header("X-Auth-Token", token)
			.body(())
			.unwrap()
			.send_async().await
			.map_err(|_| Error::Storage)?;
		//debug!("storage resp: {resp:?}");
		//debug!("storage status: {:?}", resp.status());
		match resp.status() {
			StatusCode::NO_CONTENT | StatusCode::NOT_FOUND => return Ok(()),
			_ => return Err(Error::Storage),
		}
	}
	pub async fn save(&self, data: Vec<u8>, path: &PathBuf) -> Result<PathBuf, Error> {
		let is_fixed_path = false;
		let is_force_overwrite = false;
		self._save(data, path, is_fixed_path, is_force_overwrite, false).await
	}
	pub async fn save_stream<R>(
		&self,
		reader: R,
		size_bytes: u64,
		path: &PathBuf,
	) -> Result<PathBuf, Error>
	where
		R: Read + Send + Sync + 'static,
	{
		let is_fixed_path = false;
		let is_force_overwrite = false;
		self._save_stream(reader, size_bytes, path, is_fixed_path, is_force_overwrite, false)
			.await
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
	pub async fn save_stream_fixed<R>(
		&self,
		reader: R,
		size_bytes: u64,
		path: &PathBuf,
	) -> Result<(), Error>
	where
		R: Read + Send + Sync + 'static,
	{
		let is_fixed_path = true;
		let is_force_overwrite = false;
		self._save_stream(reader, size_bytes, path, is_fixed_path, is_force_overwrite, false)
			.await
			.map(|_| ())
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

		let url = self.api_url_for(&path).await;
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

		let req = req_builder
			.body(AsyncBody::from(data))
			.map_err(|_| Error::Storage)?;
		let mut resp = req.send_async().await.map_err(|e| {
			eprintln!(
				"[saras][storage] save failed path={} err={:?}",
				path.display(),
				e
			);
			Error::Storage
		})?;
		if resp.status() != StatusCode::CREATED {
			eprintln!(
				"[saras][storage] save failed path={} status={} headers={:?}",
				path.display(),
				resp.status(),
				resp.headers()
			);
			return Err(Error::Storage);
		}
		Ok(path)
	}
	pub async fn _save_stream<R>(
		&self,
		reader: R,
		size_bytes: u64,
		path: &PathBuf,
		fixed_path: bool,
		is_force_overwrite: bool,
		is_br: bool,
	) -> Result<PathBuf, Error>
	where
		R: Read + Send + Sync + 'static,
	{
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

		let url = self.api_url_for(&path).await;
		let token = self.get_token().await;
		let is_br = is_br
			|| path
				.extension()
				.and_then(|e| e.to_str())
				.map(|s| s.eq_ignore_ascii_case("br"))
				.unwrap_or(false);

		let log_path = path.clone();
		let res: Result<(), Error> = task::spawn_blocking(move || {
			let path_str = log_path.display().to_string();
			println!(
				"[saras][storage] save_stream start path={} fixed={} force_overwrite={} br={}",
				log_path.display(),
				fixed_path,
				is_force_overwrite,
				is_br
			);
			let mut req_builder = isahc::Request::builder()
				.method("PUT")
				.uri(url)
				.header("X-Auth-Token", token);
			if is_br {
				req_builder = req_builder.header("Content-Encoding", "br");
			}
			let progress_reader = ProgressReader::new(
				reader,
				path_str,
				size_bytes,
				STREAM_PROGRESS_STEP_BYTES,
			);
			let req = req_builder
				.body(Body::from_reader_sized(progress_reader, size_bytes))
				.map_err(|_| Error::Storage)?;
			let mut resp = req.send().map_err(|e| {
				println!(
					"[saras][storage] save_stream failed path={} err={:?}",
					log_path.display(),
					e
				);
				Error::Storage
			})?;
			if resp.status() != StatusCode::CREATED {
				let status = resp.status();
				let body = resp.text().unwrap_or_else(|_| "<unreadable>".to_string());
				println!(
					"[saras][storage] save_stream failed path={} status={} body={}",
					log_path.display(),
					status,
					body
				);
				return Err(Error::Storage);
			}
			println!(
				"[saras][storage] save_stream done path={} status={}",
				log_path.display(),
				resp.status()
			);
			Ok(())
		})
		.await;

		res?;
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

	pub async fn set_container_cors(&self, cors: &ContainerCors) -> Result<(), String> {
		let conf = CONF.read().await;
		let bucket = if let Some(c) = self.container_override.as_ref() {
			c.trim().to_string()
		} else if self.use_map {
			conf.selectel.map_container_name.trim().to_string()
		} else {
			conf.selectel.container_name.trim().to_string()
		};
		let access_key = conf
			.selectel
			.s3_access_key_id
			.clone()
			.map(|s| s.trim().to_string())
			.filter(|v| !v.is_empty())
			.ok_or_else(|| "[saras] missing selectel.s3_access_key_id".to_string())?;
		let secret_key = conf
			.selectel
			.s3_secret_access_key
			.clone()
			.map(|s| s.trim().to_string())
			.filter(|v| !v.is_empty())
			.ok_or_else(|| "[saras] missing selectel.s3_secret_access_key".to_string())?;
		let api_base = if let Some(b) = self.api_base_override.as_ref() {
			b.trim().to_string()
		} else if self.use_map {
			conf.selectel.map_api_base_url.trim().to_string()
		} else {
			conf.selectel.api_base_url.trim().to_string()
		};
		let (s3_base_host, region) = s3_host_and_region_from_swift(&api_base)?;
		drop(conf);

		let host = s3_base_host.clone();
		let url = format!("https://{host}/{bucket}?cors=");
		let xml = s3_cors_xml(cors)?;
		let content_md5 = md5_base64(xml.as_bytes());

		let payload_hash = sha256_hex(xml.as_bytes());
		let (amz_date, authorization) = s3_sign_headers(
			"PUT",
			&format!("/{bucket}"),
			"cors=",
			&host,
			&payload_hash,
			Some(&content_md5),
			&access_key,
			&secret_key,
			&region,
		)?;

		let req = isahc::Request::builder()
			.method("PUT")
			.uri(url)
			.header("Host", host)
			.header("x-amz-date", amz_date)
			.header("x-amz-content-sha256", payload_hash)
			.header("Content-MD5", content_md5)
			.header("Authorization", authorization)
			.header("Content-Type", "application/xml")
			.body(AsyncBody::from(xml))
			.map_err(|e| format!("[saras] s3 cors PUT build error: {e}"))?;

		let mut resp = req
			.send_async()
			.await
			.map_err(|e| format!("[saras] s3 cors PUT send error: {e}"))?;

		let status = resp.status();
		if status == StatusCode::OK {
			return Ok(());
		}

		let mut body = resp.into_body();
		let mut bytes: Vec<u8> = Vec::new();
		body.read_to_end(&mut bytes)
			.await
			.map_err(|e| format!("[saras] s3 cors error read body: {e}"))?;
		let text = String::from_utf8_lossy(&bytes);

		Err(format!(
			"[saras] s3 cors set failed: http {} {}",
			status, text
		))
	}

	pub async fn get_container_cors(&self) -> Result<ContainerCorsState, String> {
		let conf = CONF.read().await;
		let bucket = if let Some(c) = self.container_override.as_ref() {
			c.trim().to_string()
		} else if self.use_map {
			conf.selectel.map_container_name.trim().to_string()
		} else {
			conf.selectel.container_name.trim().to_string()
		};
		let access_key = conf
			.selectel
			.s3_access_key_id
			.clone()
			.map(|s| s.trim().to_string())
			.filter(|v| !v.is_empty())
			.ok_or_else(|| "[saras] missing selectel.s3_access_key_id".to_string())?;
		let secret_key = conf
			.selectel
			.s3_secret_access_key
			.clone()
			.map(|s| s.trim().to_string())
			.filter(|v| !v.is_empty())
			.ok_or_else(|| "[saras] missing selectel.s3_secret_access_key".to_string())?;
		let api_base = if let Some(b) = self.api_base_override.as_ref() {
			b.trim().to_string()
		} else if self.use_map {
			conf.selectel.map_api_base_url.trim().to_string()
		} else {
			conf.selectel.api_base_url.trim().to_string()
		};
		let (s3_base_host, region) = s3_host_and_region_from_swift(&api_base)?;
		drop(conf);

		let host = s3_base_host.clone();
		let url = format!("https://{host}/{bucket}?cors=");

		let payload_hash = sha256_hex(b"");
		let (amz_date, authorization) = s3_sign_headers(
			"GET",
			&format!("/{bucket}"),
			"cors=",
			&host,
			&payload_hash,
			None,
			&access_key,
			&secret_key,
			&region,
		)?;

		let req = isahc::Request::builder()
			.method("GET")
			.uri(url)
			.header("Host", host)
			.header("x-amz-date", amz_date)
			.header("x-amz-content-sha256", payload_hash)
			.header("Authorization", authorization)
			.body(())
			.map_err(|e| format!("[saras] s3 cors GET build error: {e}"))?;

		let mut resp = req
			.send_async()
			.await
			.map_err(|e| format!("[saras] s3 cors GET send error: {e}"))?;

		if resp.status() != StatusCode::OK {
			let status = resp.status();
			let mut body = resp.into_body();
			let mut bytes: Vec<u8> = Vec::new();
			body.read_to_end(&mut bytes)
				.await
				.map_err(|e| format!("[saras] s3 cors error read body: {e}"))?;
			let text = String::from_utf8_lossy(&bytes);
			return Err(format!("[saras] s3 cors GET failed: http {} {}", status, text));
		}

		let mut body = resp.into_body();
		let mut bytes: Vec<u8> = Vec::new();
		body.read_to_end(&mut bytes)
			.await
			.map_err(|e| format!("[saras] s3 cors read error: {e}"))?;
		let xml = String::from_utf8(bytes).map_err(|e| format!("[saras] s3 cors utf8 error: {e}"))?;

		s3_parse_cors_xml(&xml)
	}

	pub async fn presign_put_url(
		&self,
		path: &Path,
		expires_sec: u32,
		content_type: Option<&str>,
	) -> Result<String, String> {
		let conf = CONF.read().await;
		let bucket = if let Some(c) = self.container_override.as_ref() {
			c.trim().to_string()
		} else if self.use_map {
			conf.selectel.map_container_name.trim().to_string()
		} else {
			conf.selectel.container_name.trim().to_string()
		};
		let access_key = conf
			.selectel
			.s3_access_key_id
			.clone()
			.map(|s| s.trim().to_string())
			.filter(|v| !v.is_empty())
			.ok_or_else(|| "[saras] missing selectel.s3_access_key_id".to_string())?;
		let secret_key = conf
			.selectel
			.s3_secret_access_key
			.clone()
			.map(|s| s.trim().to_string())
			.filter(|v| !v.is_empty())
			.ok_or_else(|| "[saras] missing selectel.s3_secret_access_key".to_string())?;
		let api_base = if let Some(b) = self.api_base_override.as_ref() {
			b.trim().to_string()
		} else if self.use_map {
			conf.selectel.map_api_base_url.trim().to_string()
		} else {
			conf.selectel.api_base_url.trim().to_string()
		};
		let (s3_base_host, region) = s3_host_and_region_from_swift(&api_base)?;
		drop(conf);

		let host = format!("{bucket}.{s3_base_host}");
		let key = path.to_string_lossy();
		let encoded_key = s3_uri_encode_path(&key);
		let uri = format!("/{encoded_key}");

		Ok(s3_presign_put(
			&host,
			&uri,
			expires_sec,
			content_type,
			&access_key,
			&secret_key,
			&region,
		)?)
	}
}

struct ProgressReader<R> {
	inner: R,
	path: String,
	size_bytes: u64,
	next_log: u64,
	step_bytes: u64,
	read_bytes: u64,
	next_time_log: Instant,
	time_step: StdDuration,
}

impl<R> ProgressReader<R> {
	fn new(inner: R, path: String, size_bytes: u64, step_bytes: u64) -> Self {
		let step_bytes = step_bytes.max(1);
		let first_log = STREAM_PROGRESS_FIRST_BYTES.min(step_bytes);
		let time_step = StdDuration::from_secs(STREAM_PROGRESS_TIME_SECS.max(1));
		Self {
			inner,
			path,
			size_bytes,
			next_log: first_log,
			step_bytes,
			read_bytes: 0,
			next_time_log: Instant::now() + time_step,
			time_step,
		}
	}
}

impl<R: Read> Read for ProgressReader<R> {
	fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
		let n = self.inner.read(buf)?;
		if n == 0 {
			return Ok(0);
		}
		self.read_bytes = self.read_bytes.saturating_add(n as u64);
		let now = Instant::now();
		let mut logged = false;
		while self.read_bytes >= self.next_log {
			println!(
				"[saras][storage] save_stream progress path={} bytes={}/{}",
				self.path,
				self.read_bytes,
				self.size_bytes
			);
			let _ = std::io::stdout().flush();
			logged = true;
			if self.next_log < self.step_bytes {
				self.next_log = self.step_bytes;
			} else {
				self.next_log = self.next_log.saturating_add(self.step_bytes);
			}
		}
		if !logged && now >= self.next_time_log {
			println!(
				"[saras][storage] save_stream progress path={} bytes={}/{}",
				self.path,
				self.read_bytes,
				self.size_bytes
			);
			let _ = std::io::stdout().flush();
			self.next_time_log = now + self.time_step;
		}
		Ok(n)
	}
}

type HmacSha256 = Hmac<Sha256>;

fn hex_lower(bytes: &[u8]) -> String {
	let mut out = String::with_capacity(bytes.len() * 2);
	for b in bytes {
		out.push_str(&format!("{:02x}", b));
	}
	out
}

fn hmac_sha256(key: &[u8], data: &[u8]) -> Vec<u8> {
	let mut mac = HmacSha256::new_from_slice(key).expect("HMAC can take key of any size");
	mac.update(data);
	mac.finalize().into_bytes().to_vec()
}

fn sha256_hex(data: &[u8]) -> String {
	let mut hasher = Sha256::new();
	hasher.update(data);
	hex_lower(&hasher.finalize())
}

fn s3_host_and_region_from_swift(api_base_url: &str) -> Result<(String, String), String> {
	let u = Url::parse(api_base_url).map_err(|e| format!("[saras] bad api_base_url: {e}"))?;
	let host = u
		.host_str()
		.ok_or_else(|| "[saras] api_base_url missing host".to_string())?
		.to_string();
	let s3_host = if let Some(rest) = host.strip_prefix("swift.") {
		format!("s3.{rest}")
	} else if host.starts_with("s3.") {
		host.clone()
	} else {
		return Err(format!(
			"[saras] api_base_url host must start with swift. or s3., got: {host}"
		));
	};
	let parts: Vec<&str> = s3_host.split('.').collect();
	if parts.len() < 2 {
		return Err(format!("[saras] bad s3 host: {s3_host}"));
	}
	let region = parts[1].to_string();
	Ok((s3_host, region))
}

fn s3_uri_encode_path(path: &str) -> String {
	path.split('/')
		.map(|seg| byte_serialize(seg.as_bytes()).collect::<String>())
		.collect::<Vec<_>>()
		.join("/")
}

fn s3_query_encode(s: &str) -> String {
	byte_serialize(s.as_bytes()).collect::<String>()
}

fn s3_cors_xml(cors: &ContainerCors) -> Result<String, String> {
	let allow_origins = cors
		.allow_origin
		.split(',')
		.map(|s| s.trim())
		.filter(|s| !s.is_empty())
		.collect::<Vec<_>>();
	if allow_origins.is_empty() {
		return Err("[saras] cors.allow_origin is empty".to_string());
	}
	let allow_methods = cors
		.allow_methods
		.split(',')
		.map(|s| s.trim().to_uppercase())
		.filter(|s| !s.is_empty())
		.collect::<Vec<_>>();
	if allow_methods.is_empty() {
		return Err("[saras] cors.allow_methods is empty".to_string());
	}
	for m in &allow_methods {
		match m.as_str() {
			"GET" | "PUT" | "POST" | "DELETE" | "HEAD" => {}
			_ => {
				return Err(format!(
					"[saras] invalid CORS method for S3: {m} (allowed: GET,PUT,POST,DELETE,HEAD)"
				))
			}
		}
	}
	let allow_headers = cors
		.allow_headers
		.split(',')
		.map(|s| s.trim())
		.filter(|s| !s.is_empty())
		.collect::<Vec<_>>();
	let expose_headers = cors
		.expose_headers
		.split(',')
		.map(|s| s.trim())
		.filter(|s| !s.is_empty())
		.collect::<Vec<_>>();

	let mut xml = String::new();
	xml.push_str(r#"<CORSConfiguration xmlns="http://s3.amazonaws.com/doc/2006-03-01/">"#);
	xml.push_str("<CORSRule>");
	for o in allow_origins {
		xml.push_str("<AllowedOrigin>");
		xml.push_str(o);
		xml.push_str("</AllowedOrigin>");
	}
	for m in allow_methods {
		xml.push_str("<AllowedMethod>");
		xml.push_str(&m);
		xml.push_str("</AllowedMethod>");
	}
	for h in allow_headers {
		xml.push_str("<AllowedHeader>");
		xml.push_str(h);
		xml.push_str("</AllowedHeader>");
	}
	for h in expose_headers {
		xml.push_str("<ExposeHeader>");
		xml.push_str(h);
		xml.push_str("</ExposeHeader>");
	}
	xml.push_str("<MaxAgeSeconds>");
	xml.push_str(&cors.max_age.to_string());
	xml.push_str("</MaxAgeSeconds>");
	xml.push_str("</CORSRule>");
	xml.push_str("</CORSConfiguration>");
	Ok(xml)
}

fn s3_signing_key(secret_key: &str, date_stamp: &str, region: &str) -> Vec<u8> {
	let k_date = hmac_sha256(format!("AWS4{secret_key}").as_bytes(), date_stamp.as_bytes());
	let k_region = hmac_sha256(&k_date, region.as_bytes());
	let k_service = hmac_sha256(&k_region, b"s3");
	hmac_sha256(&k_service, b"aws4_request")
}

fn s3_sign_headers(
	method: &str,
	uri: &str,
	canonical_query: &str,
	host: &str,
	payload_hash: &str,
	content_md5: Option<&str>,
	access_key: &str,
	secret_key: &str,
	region: &str,
) -> Result<(String, String), String> {
	let now = chrono::Utc::now();
	let amz_date = now.format("%Y%m%dT%H%M%SZ").to_string();
	let date_stamp = now.format("%Y%m%d").to_string();
	let (canonical_headers, signed_headers) = if let Some(md5) = content_md5 {
		(
			format!(
				"content-md5:{md5}\n\
host:{host}\n\
x-amz-content-sha256:{payload_hash}\n\
x-amz-date:{amz_date}\n"
			),
			"content-md5;host;x-amz-content-sha256;x-amz-date",
		)
	} else {
		(
			format!(
				"host:{host}\n\
x-amz-content-sha256:{payload_hash}\n\
x-amz-date:{amz_date}\n"
			),
			"host;x-amz-content-sha256;x-amz-date",
		)
	};
	let canonical_request = format!(
		"{method}\n{uri}\n{canonical_query}\n{canonical_headers}\n{signed_headers}\n{payload_hash}"
	);
	let scope = format!("{date_stamp}/{region}/s3/aws4_request");
	let string_to_sign = format!(
		"AWS4-HMAC-SHA256\n{amz_date}\n{scope}\n{}",
		sha256_hex(canonical_request.as_bytes())
	);
	let signing_key = s3_signing_key(secret_key, &date_stamp, region);
	let sig = hmac_sha256(&signing_key, string_to_sign.as_bytes());
	let signature = hex_lower(&sig);
	let authorization = format!(
		"AWS4-HMAC-SHA256 Credential={access_key}/{scope}, SignedHeaders={signed_headers}, Signature={signature}"
	);
	Ok((amz_date, authorization))
}

fn s3_presign_put(
	host: &str,
	uri: &str,
	expires_sec: u32,
	_content_type: Option<&str>,
	access_key: &str,
	secret_key: &str,
	region: &str,
) -> Result<String, String> {
	let now = chrono::Utc::now();
	let amz_date = now.format("%Y%m%dT%H%M%SZ").to_string();
	let date_stamp = now.format("%Y%m%d").to_string();
	let scope = format!("{date_stamp}/{region}/s3/aws4_request");

	let mut params: Vec<(String, String)> = Vec::new();
	params.push(("X-Amz-Algorithm".to_string(), "AWS4-HMAC-SHA256".to_string()));
	params.push(("X-Amz-Credential".to_string(), format!("{access_key}/{scope}")));
	params.push(("X-Amz-Date".to_string(), amz_date.clone()));
	params.push(("X-Amz-Expires".to_string(), expires_sec.to_string()));
	params.push(("X-Amz-SignedHeaders".to_string(), "host".to_string()));

	params.sort_by(|a, b| a.0.cmp(&b.0));
	let canonical_query = params
		.iter()
		.map(|(k, v)| format!("{}={}", s3_query_encode(k), s3_query_encode(v)))
		.collect::<Vec<_>>()
		.join("&");

	let canonical_headers = format!("host:{host}\n");
	let signed_headers = "host";
	let payload_hash = "UNSIGNED-PAYLOAD";
	let canonical_request = format!(
		"PUT\n{uri}\n{canonical_query}\n{canonical_headers}\n{signed_headers}\n{payload_hash}"
	);
	let string_to_sign = format!(
		"AWS4-HMAC-SHA256\n{amz_date}\n{scope}\n{}",
		sha256_hex(canonical_request.as_bytes())
	);
	let signing_key = s3_signing_key(secret_key, &date_stamp, region);
	let sig = hmac_sha256(&signing_key, string_to_sign.as_bytes());
	let signature = hex_lower(&sig);

	Ok(format!(
		"https://{host}{uri}?{canonical_query}&X-Amz-Signature={signature}"
	))
}

fn s3_parse_cors_xml(xml: &str) -> Result<ContainerCorsState, String> {
	let find_first = |tag: &str| -> Option<String> {
		let start = format!("<{tag}>");
		let end = format!("</{tag}>");
		let i = xml.find(&start)?;
		let j = xml[i + start.len()..].find(&end)?;
		Some(xml[i + start.len()..i + start.len() + j].to_string())
	};
	let allow_origin = find_first("AllowedOrigin");

	let mut methods: Vec<String> = Vec::new();
	let mut rest = xml;
	let start = "<AllowedMethod>";
	let end = "</AllowedMethod>";
	while let Some(i) = rest.find(start) {
		let after = &rest[i + start.len()..];
		let Some(j) = after.find(end) else { break; };
		methods.push(after[..j].to_string());
		rest = &after[j + end.len()..];
	}

	let allow_methods = if methods.is_empty() {
		None
	} else {
		Some(methods.join(","))
	};

	let allow_headers = find_first("AllowedHeader");
	let expose_headers = find_first("ExposeHeader");
	let max_age = find_first("MaxAgeSeconds").and_then(|s| s.parse::<u32>().ok());

	Ok(ContainerCorsState {
		allow_origin,
		allow_methods,
		allow_headers,
		expose_headers,
		max_age,
	})
}

fn md5_base64(data: &[u8]) -> String {
	let digest = md5::compute(data);
	base64::engine::general_purpose::STANDARD.encode(digest.0)
}
