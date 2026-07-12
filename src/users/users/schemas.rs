use serde_json::{ json, Value };
use crate::users::users::models::User;
use crate::schema::{ model_meta };
use crate::schema::schemars::{ schema_for };


pub fn user_view_schema() -> Value {
	json!(schema_for!(User))
}

pub fn admin_schemas() -> Value {
	// consent - вычисляемое поле (не в модели User), добавляем его в view-схему как boolean,
	// чтобы админка отрисовала колонку галочкой/крестом.
	let mut view = user_view_schema();
	if let Some(props) = view.get_mut("properties").and_then(|p| p.as_object_mut()) {
		props.insert(
			"consent".to_string(),
			json!({ "type": "boolean", "title": "Согласие" }),
		);
	}
	json!({
	"meta": model_meta::<User>(),
	"admin": {
		"list_fields": ["id", "email", "name", "avatar", "is_superuser", "created_at", "consent"],
		"filters": ["q"],
		"sortable_fields": crate::users::users::db::SORTABLE_FIELDS,
		"labels": { "consent": "Согласие" },
		"item_fields": ["id", "email", "name", "avatar", "is_superuser"],
	},
	"schemas": {
		"view": view,
	},
	})
}
