use std::path::PathBuf;
use std::borrow::Cow;
use std::collections::HashMap;
use serde::Deserialize;
use serde_json::{ json };
use validator::{ Validate, ValidationError };
use crate::users::users::models::User;
use crate::http::{ Request };
use crate::validation::{ ValidationErrors, parse_deser_error, make_validation_error };
use crate::users::avatars::forms::AvatarForm;
use crate::models::image_field::ImageField;



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
