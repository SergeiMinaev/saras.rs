use crate::validator::{ Validate, ValidationError };
use crate::serde::Deserialize;
use crate::serde_json;
use crate::validation::{ ValidationErrors, parse_deser_error, make_validation_error };
use crate::http::{ Request };



#[derive(Debug, Deserialize, Clone)]
pub struct PostForm {
	pub title: String,
	pub text: String,
}

impl Default for PostForm {
	fn default() -> Self {
		Self {
			title: "".to_string(),
			text: "".to_string(),
		}
	}
}



#[derive(Debug, Deserialize)]
pub struct CreatePostForm {
	#[serde(flatten)]
	pub form: PostForm,
}

impl CreatePostForm {
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
	pub fn validate(req: &Request) -> Result<PostForm, ValidationErrors> {
		match serde_json::from_str::<PostForm>(&req.body_string) {
			Err(e) => {
				return Err(parse_deser_error(e))
			},
			Ok(post_form) => {
				return Ok(post_form.clone())
				//match post_form.validate() {
				//	Err(e) => Err(e.into()),
				//	Ok(()) => Ok(post_form.clone()),
				//}
			}
		}
	}
}



#[derive(Debug, Deserialize)]
pub struct UpdatePostForm {
	#[serde(flatten)]
	pub form: PostForm,
}


impl UpdatePostForm {
	pub fn validate(req: &Request) -> Result<PostForm, ValidationErrors> {
		match serde_json::from_str::<PostForm>(&req.body_string) {
			Err(e) => {
				return Err(parse_deser_error(e))
			},
			Ok(post_form) => {
				return Ok(post_form.clone())
				//match post_form.validate() {
				//	Err(e) => Err(e.into()),
				//	Ok(()) => Ok(post_form.clone()),
				//}
			}
		}
	}
}
