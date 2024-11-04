use std::path::PathBuf;
use serde::Deserialize;
use validator::{ Validate };
use crate::http::{ Request };
use crate::validation::{ ValidationErrors, parse_deser_error };



#[derive(Debug, Validate, Deserialize, Clone)]
pub struct ImageFieldForm {
	pub path: PathBuf,
	pub data_base64: String,
}

impl ImageFieldForm {
	pub fn validate(req: &Request) -> Result<ImageFieldForm, ValidationErrors> {
		match serde_json::from_str::<ImageFieldForm>(&req.body_string) {
			Err(e) => return Err(parse_deser_error(e)),
			Ok(avatar_form) => {
				match avatar_form.validate() {
					Err(e) => Err(e.into()),
					Ok(()) => Ok(avatar_form.clone())
				}
			}
		}
	}
}
