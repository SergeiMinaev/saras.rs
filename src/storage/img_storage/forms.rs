use serde::Deserialize;
use serde_json::{ json };
use std::path::PathBuf;
use base64::engine::general_purpose;
use base64::Engine;
use validator::{ Validate };
use url::Url;
use crate::http::{ Request };
use crate::models::base::ImageStorage;
use crate::util::{norm_path, slugify};


#[derive(Debug, Validate, Deserialize, schemars::JsonSchema)]
pub struct ImgStorageUploadForm {
	#[validate(length(min = 4, max = 255))]
	pub name: String,
	pub size: usize,
	pub mimetype: String,
	pub path: String,
	pub relativePath: String,
	pub data: String,
}

impl Default for ImgStorageUploadForm {
	fn default() -> Self {
		Self {
			name: "".to_string(),
			size: 0,
			mimetype: "".to_string(),
			path: "".to_string(),
			relativePath: "".to_string(),
			data: "".to_string(),
		}
	}
}
