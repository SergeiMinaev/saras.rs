use std::borrow::Cow;
use std::collections::HashMap;
use serde::{Serialize };
use validator::ValidationError;
use validator::ValidationErrorsKind;


// Custom ValidationErrors.
// `validator::ValidationErrors` does not fit into this project due to 'static str limitations.
#[derive(Debug, Serialize, Clone)]
pub struct ValidationErrors(pub HashMap<String, ValidationError>);

impl ValidationErrors {
	pub fn new() -> Self {
		Self(HashMap::new())
	}
}


// Custom From trait to make `ValidationErrors` from `validator::ValidationErrors` .
impl From<validator::ValidationErrors> for ValidationErrors {
    fn from(input_errors: validator::ValidationErrors) -> Self {
        let mut output_errors = ValidationErrors::new();
        for (key, errs_enum) in input_errors.errors() {
			match errs_enum {
				ValidationErrorsKind::Field(errs_vec) => {
					for err in errs_vec {
						output_errors.0.insert(key.to_string(), err.clone());
					};
				},
				_ => {}
			}
        }
        output_errors
    }
}


pub fn make_validation_error(code: &str, msg: &str) -> validator::ValidationError {
	ValidationError {
        code: Cow::Owned(code.to_string()),
		message: Some(Cow::Owned(msg.to_string())),
		params: HashMap::new(),
    }
}


pub fn parse_deser_error(e: serde_json::Error) -> ValidationErrors {
	println!("validation error: {e}, done");
	let mut errs = ValidationErrors::new();
	let s = e.to_string();

	if s.contains("missing field") && s.contains('`') {
		let field = s.split('`').nth(1).unwrap_or_default();
		let msg = format!("Поле `{field}` должно быть заполнено.");
		let err = make_validation_error("missing_field", &msg);
		errs.0.insert(field.to_string(), err);
		return errs;
	}

	if s.contains("required") {
		for line in s.lines() {
			let line = line.trim();
			if line.is_empty() {
				continue;
			}

			if line.contains("required") {
				if let Some(colon_pos) = line.find(':') {
					let mut field = line[..colon_pos].trim();
					if field.starts_with('`') && field.ends_with('`') && field.len() > 1 {
						field = &field[1..field.len() - 1];
					}
					if !field.is_empty() {
						let msg = format!("Поле `{}` должно быть заполнено.", field);
						let err = make_validation_error("missing_field", &msg);
						errs.0.insert(field.to_string(), err);
						continue;
					}
				}
			}

			if let Some(idx) = line.find("Validation error: required") {
				let candidate = line[..idx].trim().trim_end_matches(':').trim();
				if !candidate.is_empty() {
					let field = candidate.trim_matches('`');
					let msg = format!("Поле `{}` должно быть заполнено.", field);
					let err = make_validation_error("missing_field", &msg);
					errs.0.insert(field.to_string(), err);
					continue;
				}
			}
		}

		if !errs.0.is_empty() {
			return errs;
		}
	}

	let err = make_validation_error("bad_type", "Одно из полей имеет неправильный тип.");
	errs.0.insert("__base".to_string(), err);
	return errs
}


pub fn validate_pwd(pwd: &str) -> Result<(), ValidationError> {
	/// Password requirements: >= 12 chars, at least one uppercase, lowercase and a digit.
    if pwd.len() < 12 {
		return Err(ValidationError::new("Пароль не должен быть короче 12 символов."))
    }

    let mut has_lowercase = false;
    for char in pwd.chars() {
        if char.is_lowercase() {
            has_lowercase = true;
            break;
        }
    }
    if !has_lowercase {
		return Err(
			ValidationError::new("Пароль должен содержать хотя бы одну букву в нижнем регистре.")
		)
    }

    let mut has_uppercase = false;
    for char in pwd.chars() {
        if char.is_uppercase() {
            has_uppercase = true;
            break;
        }
    }
    if !has_uppercase {
		return Err(
			ValidationError::new("Пароль должен содержать хотя бы одну букву в верхнем регистре.")
		)
    }

    let mut has_digit = false;
    for char in pwd.chars() {
        if char.is_numeric() {
            has_digit = true;
            break;
        }
    }
    if !has_digit {
		return Err(ValidationError::new("Пароль должен содержать хотя бы одну цифру."))
    }
	Ok(())
}
