use serde_json::{ json, Value };
use crate::schema::{ model_meta };
use crate::schema::schemars::{ schema_for };
use crate::notifications::models::Notification;

pub fn notification_view_schema() -> Value {
	json!(schema_for!(Notification))
}

pub fn admin_schemas() -> Value {
	json!({
		"meta": model_meta::<Notification>(),
		"admin": {
			"list_fields": ["id", "category", "action", "entity_type", "entity_id", "count", "read_at", "last_event_at"],
			"item_fields": ["id", "recipient_id", "category", "action", "entity_type", "entity_id", "count",
			                "last_actor_id", "payload", "created_at", "last_event_at", "read_at"],
		},
		"schemas": {
			"view": notification_view_schema(),
		},
	})
}
