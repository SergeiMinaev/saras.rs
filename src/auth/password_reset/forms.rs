use serde::Deserialize;
use validator::{Validate, ValidationError};
use crate::http::Request;
use crate::validation::{parse_json_validation, ValidationErrors};
use crate::validation;

#[derive(Debug, Validate, Deserialize, Clone)]
pub struct ResetRequestForm {
	#[validate(email, length(min = 5, max = 50))]
	pub email: String,
	#[validate(length(min = 8, max = 200), custom(function = "validate_base_url"))]
	pub base_url: String,
}

#[derive(Debug, Validate, Deserialize, Clone)]
pub struct ResetConfirmForm {
	#[validate(length(min = 32, max = 200))]
	pub token: String,
	#[validate(length(min = 12, max = 100), custom(function = "validate_pwd"))]
	pub pwd: String,
}

pub fn make_reset_request_form(req: &Request) -> Result<ResetRequestForm, ValidationErrors> {
	let form: ResetRequestForm = parse_json_validation(&req.body_string)?;
	match form.validate() {
		Err(e) => Err(e.into()),
		Ok(()) => Ok(form.clone()),
	}
}

pub fn make_reset_confirm_form(req: &Request) -> Result<ResetConfirmForm, ValidationErrors> {
	let form: ResetConfirmForm = parse_json_validation(&req.body_string)?;
	match form.validate() {
		Err(e) => Err(e.into()),
		Ok(()) => Ok(form.clone()),
	}
}

fn validate_base_url(value: &str) -> Result<(), ValidationError> {
	if value.starts_with("https://") || value.starts_with("http://") {
		return Ok(());
	}
	Err(ValidationError::new("bad_base_url"))
}

fn validate_pwd(pwd: &str) -> Result<(), ValidationError> {
	validation::validate_pwd(pwd)
}
