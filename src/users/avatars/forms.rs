use std::borrow::Cow;
use std::collections::HashMap;
use serde::Deserialize;
use serde_json::{ json };
use validator::{ Validate, ValidationError };
use crate::users::users::models::User;
use crate::http::{ Request };
use crate::validation::{ ValidationErrors, parse_deser_error, make_validation_error };



#[derive(Debug, Validate, Deserialize, Clone)]
pub struct AvatarForm {
	pub path: String,
	pub data_base64: String,
}

impl AvatarForm {
	pub fn validate(req: &Request) -> Result<AvatarForm, ValidationErrors> {
		match serde_json::from_str::<AvatarForm>(&req.body_string) {
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
