use serde::Deserialize;
use validator::Validate;
use crate::validation::{ValidationErrors, parse_json_validation};
use crate::http::Request;

#[derive(Debug, Validate, Deserialize, Clone)]
pub struct DocForm {
    #[validate(length(min = 1, max = 100))]
    pub key: String,
    #[validate(length(min = 1, max = 200))]
    pub title: String,
    pub html: String,
    #[validate(length(min = 1, max = 50))]
    pub version: String,
    pub is_active: bool,
}

pub fn make_doc_form(req: &Request) -> Result<DocForm, ValidationErrors> {
    let form: DocForm = parse_json_validation(&req.body_string)?;
    match form.validate() {
        Err(e) => Err(e.into()),
        Ok(()) => Ok(form.clone()),
    }
}
