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
	pub token_lifetime_sec: i64,
	pub api_base_url: String,
}

#[derive(Debug, Deserialize)]
pub struct Conf {
    pub socket_path: String,
    pub is_dev: bool,
	pub main_image_size: String,
	pub main_image_format: String,
	pub image_formats: Vec<String>,
	pub image_sizes: Vec<ImageSize>,

	pub selectel: SelectelConf,
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
