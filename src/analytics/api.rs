use lpsql::Lpsql;
use serde::Deserialize;
use serde_json::{json, Value};

use crate::db::get_pool;
use crate::http::{JsonResp, Request, Resp};

#[derive(Deserialize)]
struct SectionRow {
    section: String,
    today: i64,
    week: i64,
    month: i64,
    total: i64,
    today_uniq: i64,
    week_uniq: i64,
    month_uniq: i64,
    total_uniq: i64,
}

pub async fn analytics(_req: Request) -> Resp {
    let pool = get_pool();
    let q = "SELECT row_to_json(data) FROM (
        SELECT
            section,
            COUNT(*) FILTER (WHERE ts > NOW() - INTERVAL '1 day') AS today,
            COUNT(*) FILTER (WHERE ts > NOW() - INTERVAL '7 days') AS week,
            COUNT(*) FILTER (WHERE ts > NOW() - INTERVAL '30 days') AS month,
            COUNT(*) AS total,
            COUNT(DISTINCT ip) FILTER (WHERE ts > NOW() - INTERVAL '1 day') AS today_uniq,
            COUNT(DISTINCT ip) FILTER (WHERE ts > NOW() - INTERVAL '7 days') AS week_uniq,
            COUNT(DISTINCT ip) FILTER (WHERE ts > NOW() - INTERVAL '30 days') AS month_uniq,
            COUNT(DISTINCT ip) AS total_uniq
        FROM analytics_hits
        GROUP BY section
        ORDER BY total DESC
    ) data";
    let rows = Lpsql::query(q).fetch_all(&pool).await;
    let sections: Vec<Value> = rows
        .iter()
        .filter_map(|r| serde_json::from_str::<SectionRow>(r).ok())
        .map(|r| json!({
            "section": r.section,
            "today": r.today,
            "week": r.week,
            "month": r.month,
            "total": r.total,
            "today_uniq": r.today_uniq,
            "week_uniq": r.week_uniq,
            "month_uniq": r.month_uniq,
            "total_uniq": r.total_uniq,
        }))
        .collect();
    JsonResp::ok("").content(&json!({ "sections": sections })).to_http()
}
