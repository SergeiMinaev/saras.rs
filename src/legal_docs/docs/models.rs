use serde::{Serialize, Deserialize};
use crate::schemars;
use crate::models::base_model::BaseModel;
use crate::schema::textfield_schema;

#[derive(Serialize, Deserialize, Debug, schemars::JsonSchema)]
#[serde(rename_all = "lowercase")]
pub struct Doc {
    pub id: u32,
    pub key: String,
    pub title: String,
    #[schemars(schema_with = "textfield_schema")]
    pub html: String,
    pub version: String,
    pub is_active: bool,
}

impl BaseModel for Doc {
    const NAME: &'static str = "Юр. доккумент";
    const NAME_PLURAL: &'static str = "Юр. доккументы";
}
