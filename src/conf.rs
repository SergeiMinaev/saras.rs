use std::fs::File;
use std::io::Read;
use std::path::Path;
use once_cell::sync::Lazy;
use async_lock::RwLock;
use serde::{ Serialize, Deserialize };


pub static CONF: Lazy<RwLock<Conf>> = Lazy::new(|| {
    RwLock::new(Conf::new())
});


#[derive(Debug, Serialize, Deserialize)]
pub struct ImageSize {
	pub size: String,
	pub crop: bool,
}

#[derive(Debug, Deserialize)]
pub struct SelectelConf {
	pub account_id: String,
	pub proj_id: String,
	pub proj_name: String,
	pub container_name: String,
	pub container_hostname: String,
	pub svc_user_name: String,
	pub svc_user_pwd: String,
	pub s3_access_key_id: Option<String>,
	pub s3_secret_access_key: Option<String>,
	pub token_lifetime_sec: i64,
	pub api_base_url: String,
	pub map_api_base_url: String,
	pub map_container_name: String,
	pub map_container_hostname: String,
}

#[derive(Debug, Deserialize)]
pub struct LegalDocsConf {
	pub consent_key: String,
	pub consent_version: String,
}

#[derive(Debug, Deserialize)]
pub struct GeoVisitsConf {
	pub token: String,
	#[serde(default)]
	pub debug: bool,
}

#[derive(Debug, Deserialize)]
pub struct Conf {
    pub socket_path: String,
    pub is_dev: bool,
	pub static_dir: String,

	pub main_image_size: String,
	pub main_image_format: String,
	#[serde(default = "default_img_shrink_quality_mode")]
	pub img_shrink_quality_mode: String,
	pub image_formats: Vec<String>,
	pub image_sizes: Vec<ImageSize>,

	pub selectel: SelectelConf,

	pub vk_auth_url: String,
	#[serde(default)]
	pub vk_client_id: String,
	pub ya_auth_url: String,
	pub ya_auth_client_id: String,
	pub ya_auth_client_secret: String,

	pub smtp_server: String,
	pub smtp_login: String,
	pub smtp_pwd: String,

	#[serde(default)]
	pub site_name: String,

	pub legal_docs: LegalDocsConf,
	pub geo_visits: Option<GeoVisitsConf>,
}

fn default_img_shrink_quality_mode() -> String {
	"medium".to_string()
}

impl Conf {
    pub fn new() -> Self {
        let path = Path::new("saras.toml");
        let mut file = File::open(&path).unwrap();
        let mut contents = String::new();
        file.read_to_string(&mut contents).unwrap();
        let conf: Conf = toml::from_str(&contents).unwrap();
        return conf;
    }
}
