use std::sync::Arc;
use lpsql::pool::ConnectionPool;
use lpsql::Lpsql;
use serde_json::Value;
use crate::notifications::models::Notification;

/*
-- SQL (PostgreSQL)

create table if not exists notifications (
	id bigserial primary key,
	recipient_id int not null,
	category text not null,
	action text not null,
	entity_type text not null,
	entity_id text not null,
	agg_bucket timestamptz not null default date_trunc('hour', now()),
	count int not null default 1,
	last_actor_id int null,
	payload jsonb null,
	created_at timestamptz not null default now(),
	last_event_at timestamptz not null default now(),
	read_at timestamptz null
);

create unique index if not exists uniq_notifications_agg
	on notifications(recipient_id, category, action, entity_type, entity_id, agg_bucket);

create index if not exists notif_unread_idx
	on notifications(recipient_id, last_event_at desc)
	where read_at is null;
*/

pub struct NotificationDb {
	pool: Arc<ConnectionPool>,
}

impl NotificationDb {
	pub fn new(pool: Arc<ConnectionPool>) -> Self {
		Self { pool }
	}

	pub async fn page(&self, recipient_id: i32, offset: i32, size: i32) -> Vec<Notification> {
		let q = "select row_to_json(data) from (
			select
				n.id,
				n.action as label,
				n.recipient_id,
				n.category,
				n.action,
				n.entity_type,
				n.entity_id,
				n.count,
				n.last_actor_id,
				case when actor.id is not null then
					json_build_object(
						'id', actor.id,
						'label', actor.email,
						'email', actor.email,
						'name', actor.name,
						'avatar', case when actor.avatar is not null then json_build_object('path', actor.avatar) else null end
					)
				else null end as last_actor,
				n.payload,
				to_char(n.created_at, 'YYYY-MM-DD\"T\"HH24:MI:SS\"Z\"') as created_at,
				to_char(n.last_event_at, 'YYYY-MM-DD\"T\"HH24:MI:SS\"Z\"') as last_event_at,
				case when n.read_at is not null then to_char(n.read_at, 'YYYY-MM-DD\"T\"HH24:MI:SS\"Z\"') else null end as read_at
			from notifications n
			left join users_users actor on actor.id = n.last_actor_id
			where n.recipient_id = $1::INT
			order by (n.read_at is null) desc, n.last_event_at desc
			offset $2::INT limit $3::INT
		) data";
		let items = Lpsql::query(q)
			.bind(recipient_id)
			.bind(offset)
			.bind(size)
			.fetch_all(&self.pool).await;
		items.into_iter().map(|json| serde_json::from_str(&json).unwrap()).collect()
	}

	pub async fn unread_count(&self, recipient_id: i32) -> i64 {
		let q = "select count(*) from notifications
			where recipient_id = $1::INT and read_at is null";
		Lpsql::query(q).bind(recipient_id).fetch_one(&self.pool).await.unwrap().parse().unwrap()
	}

	pub async fn mark_read(&self, recipient_id: i32, ids: &[i64]) -> bool {
		let q = "update notifications
			set read_at = now()
			where recipient_id = $1::INT and id = $2::BIGINT and read_at is null
			returning id";
		let mut updated = 0;
		for id in ids {
			let r = Lpsql::query(q).bind(recipient_id).bind(*id).exec(&self.pool).await;
			updated += r;
		}
		updated > 0
	}

	pub async fn mark_all_before(&self, recipient_id: i32, ts: &str) -> i32 {
		let q = "update notifications
			set read_at = now()
			where recipient_id = $1::INT and read_at is null and last_event_at < $2::TIMESTAMPTZ";
		Lpsql::query(q).bind(recipient_id).bind(ts).exec(&self.pool).await
	}

	pub async fn upsert_aggregate(
		&self,
		recipient_id: i32,
		category: &str,
		action: &str,
		entity_type: &str,
		entity_id: &str,
		last_actor_id: Option<i32>,
		payload: Option<Value>,
	) -> Option<i64> {
		let q = "insert into notifications
			(recipient_id, category, action, entity_type, entity_id, agg_bucket, count, last_actor_id, payload)
			values ($1::INT, $2::TEXT, $3::TEXT, $4::TEXT, $5::TEXT, date_trunc('hour', now()), 1, $6::INT, $7::JSONB)
			on conflict (recipient_id, category, action, entity_type, entity_id, agg_bucket)
			do update set
				-- increment count only when the incoming actor differs from stored last_actor_id
				count = notifications.count + (
					case
						when notifications.last_actor_id is null then 1
						when excluded.last_actor_id is null then 0
						when notifications.last_actor_id <> excluded.last_actor_id then 1
						else 0
					end
				),
				last_actor_id = excluded.last_actor_id,
				last_event_at = now(),
				payload = coalesce(excluded.payload, notifications.payload)
			returning id";
		Lpsql::query(q)
			.bind(recipient_id).bind(category).bind(action)
			.bind(entity_type).bind(entity_id)
			.bind(last_actor_id)
			.bind(payload.map(|v| v.to_string()))
			.fetch_one(&self.pool).await
			.map(|id| id.parse().unwrap())
	}
}
