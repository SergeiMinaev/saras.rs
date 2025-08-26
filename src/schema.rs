use jsonschema::JSONSchema;
use serde::Serialize;
use serde_json::{ json, Value };
pub use schemars;
use schemars::schema::RootSchema;
use crate::schema::schemars::{ JsonSchema };
use crate::models::base_model::BaseModel;
use std::any::type_name;
use schemars::schema::Metadata;
use schemars::schema::Schema;



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


fn pluralize(s: &str) -> String {
    if s.ends_with('y')
        && !s.ends_with("ay")
        && !s.ends_with("ey")
        && !s.ends_with("iy")
        && !s.ends_with("oy")
        && !s.ends_with("uy")
    {
        let mut t = s[..s.len() - 1].to_string();
        t.push_str("ies");
        t
    } else if s.ends_with('s')
        || s.ends_with('x')
        || s.ends_with('z')
        || s.ends_with("ch")
        || s.ends_with("sh")
    {
        format!("{s}es")
    } else {
        format!("{s}s")
    }
}

pub fn model_meta<T: JsonSchema + BaseModel >() -> Value {
  let mut split = type_name::<T>().split("::");
  let ctg = split.nth(1).unwrap();
  let model = split.nth(2).unwrap();
  let endpoint = format!("{ctg}/{}", pluralize(&model.to_lowercase()));
  return json!({
    "endpoint": endpoint, "model_name": model,
    "name": T::NAME, "name_plural": T::NAME_PLURAL
  });
}

pub fn to_json<T: JsonSchema + BaseModel  + serde::Serialize>(m: T) -> Value {
  json!(m)
}


/// Add "description: "textfield" to the schema.
/// Usage:
/// #[derive(Serialize, Deserialize, Debug, schemars::JsonSchema)]
/// pub struct Post {
/// 	pub id: u32,
/// 	pub title: String,
/// 	#[schemars(schema_with = "textfield_schema")]
/// 	pub text: String,
/// }
pub fn textfield_schema(gen: &mut schemars::gen::SchemaGenerator) -> Schema {
    let mut schema = String::json_schema(gen);
    if let Schema::Object(ref mut obj) = schema {
        obj.metadata = Some(Box::new(Metadata {
            description: Some("textfield".to_string()),
            ..Default::default()
        }));
    }
    schema
}
