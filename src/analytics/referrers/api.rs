use lpsql::Lpsql;
use serde::Deserialize;
use serde_json::{json, Value};

use crate::db::get_pool;
use crate::http::{JsonResp, Request, Resp};

#[derive(Deserialize)]
struct HostRow {
    host: String,
    count: i64,
    uniq: i64,
}

/// Топ хостов-источников за период. `?range=today|week|month|year|all`
/// (по умолчанию month). Для админки.
pub async fn referrers(req: Request) -> Resp {
    let pool = get_pool();
    let range_cond = super::super::range_ts_cond(req.query.get("range"), "ts");
    let q = format!(
        "SELECT row_to_json(data) FROM (
        SELECT
            host,
            COUNT(*) AS count,
            COUNT(DISTINCT ip) FILTER (WHERE ip <> '') AS uniq
        FROM referrers
        WHERE {range_cond}
        GROUP BY host
        ORDER BY count DESC
        LIMIT 200
    ) data"
    );
    let rows = Lpsql::query(&q).fetch_all(&pool).await;
    let hosts: Vec<Value> = rows
        .iter()
        .filter_map(|r| serde_json::from_str::<HostRow>(r).ok())
        .map(|r| {
            json!({
                "host": r.host,
                "count": r.count,
                "uniq": r.uniq,
            })
        })
        .collect();
    JsonResp::ok("").content(&json!({ "hosts": hosts })).to_http()
}
