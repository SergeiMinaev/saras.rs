use serde_json::{ json, Value };
use crate::users::users::models::User;
use crate::schema::{ model_meta };
use crate::schema::schemars::{ schema_for };


pub fn user_view_schema() -> Value {
	json!(schema_for!(User))
}

pub fn admin_schemas() -> Value {
	json!({
	"meta": model_meta::<User>(),
	"admin": {
		"list_fields": ["id", "email", "avatar", "is_superuser"],
		"item_fields": ["id", "email", "avatar", "is_superuser"],
	},
	"schemas": {
		"view": user_view_schema(),
	},
	})
}
