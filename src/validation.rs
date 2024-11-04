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


/// Takes useful info from `serde_json::Error` and returns it as `ValidationErrors`.
pub fn parse_deser_error(e: serde_json::Error) -> ValidationErrors {
	let mut errs = ValidationErrors::new();
	let s = e.to_string();
	if s.contains("missing field") {
		// example: "missing field `age`", line: 1, column: 82
		let _ = s.split("`").nth(0).unwrap().trim().replace(" ", "_");
		let field = s.split("`").nth(1).unwrap_or_default();
		let msg = format!("Поле `{field}` должно быть заполнено.");
		let err = make_validation_error("missing_field", &msg);
		errs.0.insert(field.to_string(), err);
	} else {
		// example: "invalid type: string \"sixteen\", expected i32", line: 1, column: 80
		errs.0.insert("what".to_string(), make_validation_error("lol", "wtf"));
		let err = make_validation_error("bad_type", "Одно из полей имеет неправильный тип.");
		errs.0.insert("__base".to_string(), err);
	}
	return errs
}
