use serde::Deserialize;
use validator::Validate;

#[derive(Debug, Validate, Deserialize, schemars::JsonSchema)]
pub struct FileStorageUploadForm {
	#[validate(length(min = 1, max = 255))]
	pub name: String,
	pub size: usize,
	pub mimetype: String,
	pub path: String,
	pub relative_path: String,
	pub data: String,
}

impl Default for FileStorageUploadForm {
	fn default() -> Self {
		Self {
			name: "".to_string(),
			size: 0,
			mimetype: "".to_string(),
			path: "".to_string(),
			relative_path: "".to_string(),
			data: "".to_string(),
		}
	}
}
