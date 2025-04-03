use serde::{Serialize,Deserialize};
use crate::models::base_model::BaseModel;
use crate::models::image_field::{ ImageField };


#[derive(Serialize, Deserialize, Debug, schemars::JsonSchema)]
#[serde(rename_all = "lowercase")]
pub struct Profile {
	pub label: String,
	pub name: String,
	pub avatar: Option<ImageField>,
	pub default_avatar: Option<ImageField>,
}

impl BaseModel for Profile {
	const NAME: &'static str = "Профиль";
	const NAME_PLURAL: &'static str = "Профили";
}
