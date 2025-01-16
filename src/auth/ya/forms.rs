use serde::Deserialize;
use serde_json;
use validator::{ Validate };
use crate::http::{ Request };
use crate::validation::{ ValidationErrors, parse_deser_error };



#[derive(Debug, Validate, Deserialize, Clone)]
pub struct AuthVkForm {
	pub code: String,
}

impl AuthVkForm {
	pub fn validate(req: &Request) -> Result<AuthVkForm, ValidationErrors> {
		match serde_json::from_str::<AuthVkForm>(&req.body_string) {
			Err(e) => return Err(parse_deser_error(e)),
			Ok(form) => {
				match form.validate() {
					Err(e) => Err(e.into()),
					Ok(()) => Ok(form.clone())
				}
			}
		}
	}
}
