use lpsql::Lpsql;
use crate::db::get_pool;

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
