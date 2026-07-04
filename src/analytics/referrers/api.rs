use lpsql::Lpsql;
use serde::Deserialize;
use serde_json::{json, Value};

use crate::db::get_pool;
use crate::http::{not_found, JsonResp, Request, Resp};

/// Приём «маячка» с внешним источником перехода. Фронт присылает
/// `document.referrer`; хост извлекаем и фильтруем на сервере, путь/query
/// отбрасываем. Ответ всегда ok - это fire-and-forget с клиента.
pub async fn record(req: Request) -> Resp {
    if req.method.to_lowercase() != "post" {
        return not_found();
    }

    let body: Value = serde_json::from_str(&req.body_string).unwrap_or(Value::Null);
    let referrer = body.get("referrer").and_then(|v| v.as_str()).unwrap_or("");

    if let Some(host) = super::host_from_referrer(referrer) {
        let ip = req
            .headers
            .get("x-real-ip")
            .cloned()
            .or_else(|| {
                req.headers
                    .get("x-forwarded-for")
                    .and_then(|v| v.split(',').next().map(|s| s.trim().to_string()))
            })
            .filter(|v| !v.is_empty())
            .unwrap_or_default();
        super::track(host, ip);
    }

    JsonResp::ok("").to_http()
}

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
