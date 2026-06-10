use lpsql::Lpsql;
use serde::Deserialize;
use serde_json::{json, Value};

use crate::db::get_pool;
use crate::http::{JsonResp, Request, Resp};

#[derive(Deserialize)]
struct GeoPoint {
    lat: f64,
    lon: f64,
    city: Option<String>,
    country: Option<String>,
    unique_visitors: i64,
    count_404: i64,
    count_authed: i64,
}

pub async fn geo_visits(_req: Request) -> Resp {
    let pool = get_pool();
    let q = "SELECT row_to_json(data) FROM (
        SELECT
            c.lat,
            c.lon,
            c.city,
            c.country,
            COUNT(DISTINCT v.dedup_key) AS unique_visitors,
            COUNT(*) FILTER (WHERE v.is_404) AS count_404,
            COUNT(*) FILTER (WHERE v.is_authed) AS count_authed
        FROM geo_visits v
        JOIN geo_prefix_cache c ON c.prefix = v.prefix
        WHERE v.ts > now() - interval '30 days'
          AND c.lat IS NOT NULL
        GROUP BY c.lat, c.lon, c.city, c.country
        ORDER BY unique_visitors DESC
    ) data";
    let rows = Lpsql::query(q).fetch_all(&pool).await;
    let points: Vec<Value> = rows
        .iter()
        .filter_map(|r| serde_json::from_str::<GeoPoint>(r).ok())
        .map(|p| {
            json!({
                "lat": p.lat,
                "lon": p.lon,
                "city": p.city,
                "country": p.country,
                "unique_visitors": p.unique_visitors,
                "count_404": p.count_404,
                "count_authed": p.count_authed,
            })
        })
        .collect();
    JsonResp::ok("").content(&json!({ "points": points })).to_http()
}
