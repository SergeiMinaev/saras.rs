use serde::Deserialize;
use validator::{ Validate, ValidationError };
use crate::http::{ Request };
use crate::validation::{ ValidationErrors, parse_json_validation };
use crate::forms::image_field_form::ImageFieldForm;
use crate::validation;
use crate::users::users::db::UserDb;
use crate::db::get_pool;
use futures_lite::future;



#[derive(Debug, Validate, Deserialize, Clone)]
pub struct RegForm {
	#[validate(length(min = 3, max=30), custom(function = "validate_name"))]
	pub name: String,
	#[validate(email, length(min = 5, max=50), custom(function = "validate_email"))]
	pub email: String,
	#[validate(length(min = 12, max=100), custom(function = "validate_pwd"))]
	pub pwd: String,
	pub code: Option<String>,
}


pub fn make_regform(req: &Request) -> Result<RegForm, ValidationErrors> {
	let form: RegForm = parse_json_validation(&req.body_string)?;
	match form.validate() {
		Err(e) => Err(e.into()),
		Ok(()) => Ok(form.clone()),
	}
}

fn validate_email(email: &str) ->  Result<(), ValidationError> {
	future::block_on(avalidate_email(email))
}

async fn avalidate_email(email: &str) ->  Result<(), ValidationError> {
	let pool = get_pool();
	let userdb = UserDb::new(pool.clone());
	if userdb.by_email(email).await.is_some() {
		return Err(ValidationError::new("Пользователь с таким email уже зарегистрирован."))
	}
	Ok(())
}

fn validate_name(_email: &str) ->  Result<(), ValidationError> {
	Ok(())
}

fn validate_pwd(pwd: &str) ->  Result<(), ValidationError> {
	validation::validate_pwd(pwd)
}
