use serde::Deserialize;
use validator::Validate;
use crate::http::{ Request };
use crate::validation::{ ValidationErrors, parse_json_validation };



#[derive(Debug, Validate, Deserialize, Clone)]
pub struct AvatarForm {
	pub path: String,
	pub data_base64: String,
}

impl AvatarForm {
	pub fn validate(req: &Request) -> Result<AvatarForm, ValidationErrors> {
		let avatar_form: AvatarForm = parse_json_validation(&req.body_string)?;
		match avatar_form.validate() {
			Err(e) => Err(e.into()),
			Ok(()) => Ok(avatar_form.clone()),
		}
	}
}
