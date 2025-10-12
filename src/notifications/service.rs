use serde_json::Value;
use crate::errors::Error;
use crate::db::get_pool;
use crate::notifications::db::NotificationDb;

pub async fn notify_like(
	recipient_id: i32,
	actor_id: i32,
	entity_type: &str,
	entity_id: &str,
	payload: Option<Value>,
) -> Result<(), Error> {
	println!("notify_like");
	if recipient_id == actor_id { return Ok(()) }
	let pool = get_pool();
	let db = NotificationDb::new(pool.clone());
	let _ = db.upsert_aggregate(
		recipient_id, "social", "like", entity_type, entity_id, Some(actor_id), payload
	).await;
	Ok(())
}

pub async fn notify_custom(
	recipient_id: i32,
	category: &str,
	action: &str,
	entity_type: &str,
	entity_id: &str,
	actor_id: Option<i32>,
	payload: Option<Value>,
) -> Result<(), Error> {
	println!("notify_custom");
	let pool = get_pool();
	let db = NotificationDb::new(pool.clone());
	let _ = db.upsert_aggregate(
		recipient_id, category, action, entity_type, entity_id, actor_id, payload
	).await;
	Ok(())
}

pub async fn list(recipient_id: i32, offset: i32, size: i32) -> Vec<crate::notifications::models::Notification> {
	let pool = get_pool();
	let db = NotificationDb::new(pool.clone());
	db.page(recipient_id, offset, size).await
}

pub async fn unread_count(recipient_id: i32) -> i64 {
	let pool = get_pool();
	let db = NotificationDb::new(pool.clone());
	db.unread_count(recipient_id).await
}

pub async fn mark_read(recipient_id: i32, ids: Vec<i64>) -> bool {
	let pool = get_pool();
	let db = NotificationDb::new(pool.clone());
	db.mark_read(recipient_id, &ids).await
}

pub async fn mark_all_before(recipient_id: i32, ts: &str) -> i32 {
	let pool = get_pool();
	let db = NotificationDb::new(pool.clone());
	db.mark_all_before(recipient_id, ts).await
}
