use schemars::schema::{ Schema, SchemaObject, Metadata };
use schemars::{gen::SchemaGenerator, JsonSchema};
use serde::{ Serialize,Deserialize };




#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ImageField {
  pub path: String,
}

impl JsonSchema for ImageField {
    fn schema_name() -> String { "ImageField".to_owned() }

    fn json_schema(gen: &mut SchemaGenerator) -> Schema {
        let mut schema: SchemaObject = <String>::json_schema(gen).into();
        schema.format = Some("image".to_owned());
        schema.metadata = Some(Box::new(Metadata {
            //description: Some("Path to image file.".to_owned()),
            //examples: vec![json!("SAMPLE")],
            ..Default::default()
        }));
        schema.into()
    }

    fn is_referenceable() -> bool { false }
}
