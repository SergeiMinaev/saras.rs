use crate::serde_json::{ json, Value };
use crate::posts::posts::models::Post;
use crate::schema::{ model_meta };
use crate::schema::schemars::{ schema_for };


pub fn post_view_schema() -> Value {
	json!(schema_for!(Post))
}

pub fn admin_schemas() -> Value {
	json!({
	"meta": model_meta::<Post>(),
	"admin": {
		"list_fields": ["id", "title", "text"],
		"item_fields": ["id", "title", "text"],
	},
	"schemas": {
		"view": post_view_schema(),
	},
	})
}
