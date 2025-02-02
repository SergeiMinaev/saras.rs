use serde::Deserialize;
use serde_json;
use validator::{ Validate, ValidationError };
use crate::http::{ Request };
use crate::validation::{ ValidationErrors, parse_deser_error, make_validation_error };
use crate::forms::image_field_form::ImageFieldForm;



#[derive(Debug, Validate, Deserialize, Clone)]
pub struct UserForm {
	#[validate(length(min = 5), custom(function = "validate_email"))]
	pub email: String,
	#[validate(length(min = 10))]
	pub pwd: Option<String>,
	#[serde(default)]
	pub is_superuser: Option<bool>,
	pub avatar: Option<ImageFieldForm>,
	pub name: Option<String>,
}

impl Default for UserForm {
	fn default() -> Self {
		Self {
			email: "".to_string(),
			pwd: None,
			is_superuser: Some(false),
			avatar: None,
			name: None,
		}
	}
}

//fn get_false() -> Option<bool> { Some(false) }

fn validate_email(_email: &str) ->  Result<(), ValidationError> {
	//Err(ValidationError::new("terrible email"))
	Ok(())
}


#[derive(Debug, Deserialize)]
pub struct CreateUserForm {
	#[serde(flatten)]
	pub form: UserForm,
}

impl CreateUserForm {
	pub fn validate_pwd(pwd: &Option<String>) -> Result<(), ValidationErrors> {
		if pwd.is_some() == false {
			let mut errs = ValidationErrors::new();
			errs.0.insert(
				"pwd".to_string(),
				make_validation_error("required", "Поле `пароль` должно быть заполнено.")
			);
			return Err(errs)
		}
		Ok(())
	}
	pub fn validate(req: &Request) -> Result<UserForm, ValidationErrors> {
		match serde_json::from_str::<UserForm>(&req.body_string) {
			Err(e) => {
				return Err(parse_deser_error(e))
			},
			Ok(user_form) => {
				match user_form.validate() {
					Err(e) => Err(e.into()),
					Ok(()) => {
						match CreateUserForm::validate_pwd(&user_form.pwd) {
							Err(e) => Err(e),
							Ok(()) => Ok(user_form.clone())
						}
					},
				}
			}
		}
	}
}


#[derive(Debug, Deserialize)]
pub struct UpdateUserForm {
	#[serde(flatten)]
	pub form: UserForm,
}


impl UpdateUserForm {
	pub fn validate(req: &Request) -> Result<UserForm, ValidationErrors> {
		match serde_json::from_str::<UserForm>(&req.body_string) {
			Err(e) => {
				return Err(parse_deser_error(e))
			},
			Ok(user_form) => {
				match user_form.validate() {
					Err(e) => Err(e.into()),
					Ok(()) => Ok(user_form.clone()),
				}
			}
		}
	}
}
