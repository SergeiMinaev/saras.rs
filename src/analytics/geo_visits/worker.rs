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

fn is_debug() -> bool {
    let conf = CONF.try_read();
    conf.map(|c| c.geo_visits.as_ref().map(|g| g.debug).unwrap_or(false)).unwrap_or(false)
}

async fn process(record: VisitRecord) {
    if record.ip.is_empty() {
        return;
    }
    if is_debug() {
        println!("geo_visits: process ip={}", record.ip);
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
    let (token, debug) = {
        let conf = CONF.read().await;
        let gv = conf.geo_visits.as_ref()?;
        (gv.token.clone(), gv.debug)
    };
    let url = format!("https://api.2ip.io/{}?token={}", ip, token);
    if debug {
        println!("geo_visits: lookup url={}", url);
    }
    let mut resp = match isahc::get_async(&url).await {
        Ok(r) => r,
        Err(e) => {
            if debug {
                println!("geo_visits: http error for {}: {:?}", ip, e);
            }
            return None;
        }
    };
    if !resp.status().is_success() {
        if debug {
            println!("geo_visits: non-200 for {}: {}", ip, resp.status());
        }
        return None;
    }
    let body = match resp.text().await {
        Ok(b) => b,
        Err(e) => {
            if debug {
                println!("geo_visits: read body error for {}: {:?}", ip, e);
            }
            return None;
        }
    };
    match serde_json::from_str::<TwoIpResponse>(&body) {
        Ok(parsed) => {
            let lat = parsed.lat.as_deref().and_then(|s| s.parse::<f64>().ok());
            let lon = parsed.lon.as_deref().and_then(|s| s.parse::<f64>().ok());
            if debug {
                println!("geo_visits: lookup ok ip={} lat={:?} lon={:?}", ip, lat, lon);
            }
            Some(GeoResult { lat, lon, country: parsed.country, city: parsed.city })
        }
        Err(e) => {
            if debug {
                println!("geo_visits: json error for {}: {:?} body={}", ip, e, &body[..body.len().min(300)]);
            }
            None
        }
    }
}
