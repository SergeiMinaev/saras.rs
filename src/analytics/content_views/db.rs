use lpsql::Lpsql;
use serde::Deserialize;

use crate::db::get_pool;

pub async fn insert_view(kind: &str, item_id: i64, ip: &str) {
    let pool = get_pool();
    let q = "INSERT INTO content_views (kind, item_id, ip) VALUES ($1, $2, $3)";
    let _ = Lpsql::query(q)
        .bind(kind)
        .bind(item_id)
        .bind(ip)
        .exec(&pool)
        .await;
}

#[derive(Debug, Deserialize)]
pub struct TopView {
    pub kind: String,
    pub item_id: i64,
    pub count: i64,
    pub uniq: i64,
}

/// Топ материалов по просмотрам за период — generic-агрегат без заголовков.
/// `range`: today|week|month|year|all (по умолчанию month). Заголовки
/// подставляет проектный слой, джойня item_id на свои таблицы.
pub async fn top_views(range: Option<&String>, limit: i64) -> Vec<TopView> {
    let pool = get_pool();
    let range_cond = super::super::range_ts_cond(range, "ts");
    let q = format!(
        "SELECT row_to_json(data) FROM (
        SELECT
            kind,
            item_id,
            COUNT(*) AS count,
            COUNT(DISTINCT ip) FILTER (WHERE ip <> '') AS uniq
        FROM content_views
        WHERE {range_cond}
        GROUP BY kind, item_id
        ORDER BY count DESC
        LIMIT {limit}
    ) data"
    );
    let rows = Lpsql::query(&q).fetch_all(&pool).await;
    rows.iter()
        .filter_map(|r| serde_json::from_str(r).ok())
        .collect()
}
