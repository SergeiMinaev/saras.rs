use crate::serde::Deserialize;
use crate::validation::{field_error, ValidationErrors, parse_json_validation};
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
			return Err(field_error("pwd", "required", "Поле `пароль` должно быть заполнено."))
		}
		Ok(())
	}
	pub fn validate(req: &Request) -> Result<PostForm, ValidationErrors> {
		let post_form: PostForm = parse_json_validation(&req.body_string)?;
		Ok(post_form.clone())
	}
}



#[derive(Debug, Deserialize)]
pub struct UpdatePostForm {
	#[serde(flatten)]
	pub form: PostForm,
}


impl UpdatePostForm {
	pub fn validate(req: &Request) -> Result<PostForm, ValidationErrors> {
		let post_form: PostForm = parse_json_validation(&req.body_string)?;
		Ok(post_form.clone())
	}
}
