use jsonschema::JSONSchema;
use serde::Serialize;
use serde_json::{ json, Value };
pub use schemars;
use schemars::schema::RootSchema;
use crate::schema::schemars::{ JsonSchema };
use crate::models::BaseModel;
use std::any::type_name;



#[derive(Serialize, Debug)]
pub struct ValidationError {
    pub error: String,
    pub path: String,
}

#[derive(Serialize, Debug)]
pub struct ValidationResult {
    pub is_valid: bool,
    pub errors: Value,
}

pub fn validate(schema: &RootSchema, data: &Value) -> ValidationResult {
    let schema: Value = serde_json::from_str(&serde_json::to_string(&schema).unwrap()).unwrap();
    let compiled = JSONSchema::compile(&schema).expect("A valid schema");
    let result = compiled.validate(&data);
    let mut errors: Vec<ValidationError> = vec![];
    if let Err(errs) = result {
        for err in errs {
            errors.push( ValidationError {
                error: format!("{}", err), path: format!("{}", err.instance_path)
            });
        }
    }
    let is_valid = if errors.len() == 0 { true } else { false };
    return ValidationResult { is_valid: is_valid, errors: json!(errors) }
}


pub fn model_meta<T: JsonSchema + BaseModel >() -> Value {
  let mut split = type_name::<T>().split("::");
  let ctg = split.nth(1).unwrap();
  let model = split.nth(2).unwrap();
  let endpoint = format!("{ctg}/{}", model.to_lowercase() + "s");
  return json!({
    "endpoint": endpoint, "model_name": model,
    "name": T::NAME, "name_plural": T::NAME_PLURAL
  });
}

pub fn to_json<T: JsonSchema + BaseModel  + serde::Serialize>(m: T) -> Value {
  json!(m)
}
