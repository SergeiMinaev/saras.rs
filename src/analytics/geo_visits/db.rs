use lpsql::Lpsql;
use serde::Deserialize;

use crate::db::get_pool;

#[derive(Debug, Deserialize)]
pub struct CachedPrefix {
    pub lat: Option<f64>,
    pub lon: Option<f64>,
    pub country: Option<String>,
    pub city: Option<String>,
}

pub async fn get_cached_prefix(prefix: &str) -> Option<CachedPrefix> {
    let pool = get_pool();
    let q = "SELECT row_to_json(data) FROM (
        SELECT lat, lon, country, city FROM geo_prefix_cache WHERE prefix = $1
    ) data";
    let rows = Lpsql::query(q).bind(prefix).fetch_all(&pool).await;
    rows.first().and_then(|r| serde_json::from_str(r).ok())
}

pub async fn set_cached_prefix(
    prefix: &str,
    lat: Option<f64>,
    lon: Option<f64>,
    country: Option<&str>,
    city: Option<&str>,
) {
    let pool = get_pool();
    let q = "INSERT INTO geo_prefix_cache (prefix, lat, lon, country, city)
             VALUES ($1, $2, $3, $4, $5)
             ON CONFLICT (prefix) DO NOTHING";
    let _ = Lpsql::query(q)
        .bind(prefix)
        .bind(lat)
        .bind(lon)
        .bind(country)
        .bind(city)
        .exec(&pool)
        .await;
}

pub async fn insert_visit(prefix: &str, is_404: bool, is_authed: bool, dedup_key: &str) {
    let pool = get_pool();
    let q = "INSERT INTO geo_visits (prefix, is_404, is_authed, dedup_key)
             VALUES ($1, $2, $3, $4)
             ON CONFLICT (prefix, dedup_key, day) DO NOTHING";
    let _ = Lpsql::query(q)
        .bind(prefix)
        .bind(is_404)
        .bind(is_authed)
        .bind(dedup_key)
        .exec(&pool)
        .await;
}
