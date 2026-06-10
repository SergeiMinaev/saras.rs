use async_channel::Receiver;
use isahc::AsyncReadResponseExt;
use serde::Deserialize;

use crate::conf::CONF;

use super::{db, dedup_key, ip_to_prefix, VisitRecord};

#[derive(Deserialize)]
struct TwoIpResponse {
    city: Option<String>,
    lat: Option<String>,
    lon: Option<String>,
    country: Option<String>,
    code: Option<String>,
}

pub async fn run(rx: Receiver<VisitRecord>) {
    println!("geo_visits_worker: started");
    while let Ok(record) = rx.recv().await {
        process(record).await;
    }
}

async fn process(record: VisitRecord) {
    if record.ip.is_empty() {
        return;
    }
    let prefix = ip_to_prefix(&record.ip);
    let cached = db::get_cached_prefix(&prefix).await;
    let geo = match cached {
        Some(c) => c,
        None => {
            let Some(geo) = lookup(&record.ip).await else { return };
            db::set_cached_prefix(
                &prefix,
                geo.lat,
                geo.lon,
                geo.country.as_deref(),
                geo.city.as_deref(),
            )
            .await;
            db::CachedPrefix {
                lat: geo.lat,
                lon: geo.lon,
                country: geo.country,
                city: geo.city,
            }
        }
    };
    if geo.lat.is_none() {
        return;
    }
    let key = if !record.session_id.is_empty() {
        dedup_key(&record.session_id)
    } else {
        dedup_key(&record.ip)
    };
    let is_authed = !record.session_id.is_empty();
    db::insert_visit(&prefix, record.is_404, is_authed, &key).await;
}

struct GeoResult {
    lat: Option<f64>,
    lon: Option<f64>,
    country: Option<String>,
    city: Option<String>,
}

async fn lookup(ip: &str) -> Option<GeoResult> {
    let token = {
        let conf = CONF.read().await;
        conf.geo_visits.as_ref()?.token.clone()
    };
    let url = format!("https://api.2ip.io/{}?token={}", ip, token);
    let mut resp = isahc::get_async(&url).await.ok()?;
    if !resp.status().is_success() {
        return None;
    }
    let body = resp.text().await.ok()?;
    let parsed: TwoIpResponse = serde_json::from_str(&body).ok()?;
    let lat = parsed.lat.as_deref().and_then(|s| s.parse::<f64>().ok());
    let lon = parsed.lon.as_deref().and_then(|s| s.parse::<f64>().ok());
    Some(GeoResult {
        lat,
        lon,
        country: parsed.country,
        city: parsed.city,
    })
}
