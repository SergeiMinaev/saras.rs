use lpsql::Lpsql;
use crate::db::get_pool;
use crate::conf::CONF;
use std::collections::HashSet;
use serde_json::{json, Value};

pub async fn create_consent(
    user_id: i32,
    doc_key: &str,
    doc_version: &str,
    source: &str,
    ip: Option<String>,
    user_agent: Option<String>,
) -> bool {
    let pool = get_pool();
    let q = "insert into user_consents
        (user_id, doc_key, doc_version, accepted_at, source, ip, user_agent)
        values ($1::INT, $2::TEXT, $3::TEXT, now(), $4::TEXT, $5::TEXT, $6::TEXT)
        on conflict do nothing";
    Lpsql::query(q)
        .bind(user_id)
        .bind(doc_key)
        .bind(doc_version)
        .bind(source)
        .bind(ip.unwrap_or_default())
        .bind(user_agent.unwrap_or_default())
        .exec(&pool).await > 0
}

/// Есть ли у пользователя согласие именно текущей версии документа.
pub async fn has_current_consent(user_id: i32, doc_key: &str, doc_version: &str) -> bool {
    let pool = get_pool();
    let q = "select count(*) from user_consents
        where user_id = $1::INT and doc_key = $2::TEXT and doc_version = $3::TEXT";
    let n: i64 = Lpsql::query(q)
        .bind(user_id)
        .bind(doc_key)
        .bind(doc_version)
        .fetch_one(&pool)
        .await
        .and_then(|v| v.parse().ok())
        .unwrap_or(0);
    n > 0
}

/// Блок состояния согласия для бутстрап-ответов приложений (`core-data`, `/api/user`).
/// `needs = true` → пользователю нужно принять согласие текущей версии (ключ/версия из конфига).
pub async fn consent_block(user_id: i32) -> Value {
    let (key, version) = {
        let conf = CONF.read().await;
        (
            conf.legal_docs.consent_key.clone(),
            conf.legal_docs.consent_version.clone(),
        )
    };
    let has = has_current_consent(user_id, &key, &version).await;
    json!({
        "needs": !has,
        "doc_key": key,
        "doc_version": version,
    })
}

/// Из переданного списка user_id возвращает те, у кого есть согласие указанной версии.
/// id - целые из БД, поэтому безопасно подставляются в IN напрямую.
pub async fn consented_user_ids(doc_key: &str, doc_version: &str, ids: &[i32]) -> HashSet<i32> {
    if ids.is_empty() {
        return HashSet::new();
    }
    let pool = get_pool();
    let ids_csv = ids
        .iter()
        .map(|i| i.to_string())
        .collect::<Vec<_>>()
        .join(",");
    let q = format!(
        "select row_to_json(t) from (
            select distinct user_id from user_consents
            where doc_key = $1::TEXT and doc_version = $2::TEXT
            and user_id in ({ids_csv})
        ) t"
    );
    let rows = Lpsql::query(&q)
        .bind(doc_key)
        .bind(doc_version)
        .fetch_all(&pool)
        .await;
    rows.iter()
        .filter_map(|j| serde_json::from_str::<Value>(j).ok())
        .filter_map(|v| v.get("user_id").and_then(|x| x.as_i64()))
        .map(|n| n as i32)
        .collect()
}
