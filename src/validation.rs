use std::borrow::Cow;
use std::collections::HashMap;
use serde::de::DeserializeOwned;
use serde::Serialize;
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

	pub fn push_field(&mut self, field: &str, err: ValidationError) {
		self.0.insert(field.to_string(), err);
	}

	pub fn push_base(&mut self, err: ValidationError) {
		self.0.insert("__base".to_string(), err);
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

pub fn field_error(field: &str, code: &str, msg: &str) -> ValidationErrors {
	let mut errs = ValidationErrors::new();
	errs.push_field(field, make_validation_error(code, msg));
	errs
}

pub fn base_error(code: &str, msg: &str) -> ValidationErrors {
	let mut errs = ValidationErrors::new();
	errs.push_base(make_validation_error(code, msg));
	errs
}


pub fn parse_json_validation<T: DeserializeOwned>(body: &str) -> Result<T, ValidationErrors> {
	let mut deserializer = serde_json::Deserializer::from_str(body);
	serde_path_to_error::deserialize(&mut deserializer).map_err(parse_deser_error)
}

pub fn parse_deser_error(e: serde_path_to_error::Error<serde_json::Error>) -> ValidationErrors {
	let mut errs = ValidationErrors::new();
	let path = e.path().to_string();
	let inner = e.inner();
	let s = inner.to_string();
	let field = if path.is_empty() || path == "." {
		"__base".to_string()
	} else {
		path
	};

	if inner.is_syntax() || inner.is_eof() {
		let err = make_validation_error("bad_json", "Некорректный JSON.");
		errs.0.insert("__base".to_string(), err);
		return errs;
	}

	if s.contains("missing field") {
		let field_name = if field == "__base" {
			s.split('`').nth(1).unwrap_or_default().to_string()
		} else {
			field.clone()
		};
		let target = if field_name.is_empty() {
			"__base".to_string()
		} else {
			field_name
		};
		let msg = if target == "__base" {
			"Поле должно быть заполнено.".to_string()
		} else {
			format!("Поле `{}` должно быть заполнено.", target)
		};
		let err = make_validation_error("missing_field", &msg);
		errs.0.insert(target, err);
		return errs;
	}

	let err = make_validation_error("bad_type", "Одно из полей имеет неправильный тип.");
	errs.0.insert(field, err);
	errs
}


// Требование к паролю — только минимальная длина (12 символов). Композиционные
// правила (обязательные регистры/цифры) сознательно убраны по NIST SP 800-63B:
// они дают предсказуемые пароли и запрещают стойкие парольные фразы.
// Длина — в символах, не в байтах (иначе кириллица считалась бы вдвое длиннее).
pub fn validate_pwd(pwd: &str) -> Result<(), ValidationError> {
    if pwd.chars().count() < 12 {
        return Err(ValidationError::new("Пароль не должен быть короче 12 символов."));
    }
    Ok(())
}
