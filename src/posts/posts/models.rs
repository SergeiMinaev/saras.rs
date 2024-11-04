use serde::{Serialize,Deserialize};
use crate::schemars;
use crate::models::base_model::BaseModel;
use crate::schema::textfield_schema;


#[derive(Serialize, Deserialize, Debug, schemars::JsonSchema)]
#[serde(rename_all = "lowercase")]
pub struct Post {
	pub id: u32,
	pub title: String,
	#[schemars(schema_with = "textfield_schema")]
	pub text: String,
}

impl BaseModel for Post {
	const NAME: &'static str = "Публикация";
	const NAME_PLURAL: &'static str = "Публикации";
}
