use serde::Deserialize;
use validator::{ Validate };
use crate::http::{ Request };
use crate::validation::{ ValidationErrors, parse_json_validation };



#[derive(Debug, Validate, Deserialize, Clone)]
pub struct AuthVkForm {
	pub code: String,
}

impl AuthVkForm {
	pub fn validate(req: &Request) -> Result<AuthVkForm, ValidationErrors> {
		let form: AuthVkForm = parse_json_validation(&req.body_string)?;
		match form.validate() {
			Err(e) => Err(e.into()),
			Ok(()) => Ok(form.clone()),
		}
	}
}
