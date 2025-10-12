use serde::{Serialize, Deserialize};
use serde_json::Value;
use crate::models::base_model::BaseModel;

#[derive(Serialize, Deserialize, Debug, schemars::JsonSchema)]
pub struct Notification {
	pub id: u32,
	pub label: String,
	pub recipient_id: u32,
	pub category: String,
	pub action: String,
	pub entity_type: String,
	pub entity_id: String,
	pub count: i32,
	// keep raw id for internal use
	pub last_actor_id: Option<u32>,
	// human-readable actor preview joined from users_users (null when absent)
	pub last_actor: Option<Value>,
	pub payload: Option<Value>,
	pub created_at: String,
	pub last_event_at: String,
	pub read_at: Option<String>,
}

impl BaseModel for Notification {
	const NAME: &'static str = "Уведомление";
	const NAME_PLURAL: &'static str = "Уведомления";
}
