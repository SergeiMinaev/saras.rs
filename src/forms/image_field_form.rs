use std::path::PathBuf;
use serde::Deserialize;
use validator::{ Validate };
use crate::http::{ Request };
use crate::validation::{ ValidationErrors, parse_json_validation };



#[derive(Debug, Validate, Deserialize, Clone)]
pub struct ImageFieldForm {
	/// Относительный путь до файла
	pub path: PathBuf,
	/// Содержимое файла в base64
	pub data_base64: Option<String>,
	/// Если true, файл будет удалён.
	pub del: Option<bool>,
}

impl ImageFieldForm {
	pub fn validate(req: &Request) -> Result<ImageFieldForm, ValidationErrors> {
		let avatar_form: ImageFieldForm = parse_json_validation(&req.body_string)?;
		match avatar_form.validate() {
			Err(e) => Err(e.into()),
			Ok(()) => Ok(avatar_form.clone()),
		}
	}
}
