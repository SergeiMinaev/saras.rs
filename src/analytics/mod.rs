pub mod api;
pub mod content_views;
pub mod geo_visits;
pub mod referrers;
pub mod worker;

use async_channel::{bounded, Receiver, Sender};
use once_cell::sync::OnceCell;

pub struct HitRecord {
    pub ip: String,
    pub path: String,
    pub section: String,
}

static SENDER: OnceCell<Sender<HitRecord>> = OnceCell::new();

pub fn init() -> Receiver<HitRecord> {
    let (tx, rx) = bounded(2000);
    let _ = SENDER.set(tx);
    rx
}

pub fn record(ip: &str, path: &str) {
    let Some(tx) = SENDER.get() else { return };
    let _ = tx.try_send(HitRecord {
        ip: ip.to_string(),
        path: path.to_string(),
        section: extract_section(path).to_string(),
    });
}

/// SQL-условие по диапазону дат для колонки `col`.
///
/// Значения `range`: today | week | month | year | all. По умолчанию (None или
/// неизвестное) — month. `col` задаётся вызывающим кодом (не пользователем),
/// а интервалы захардкожены, поэтому SQL-инъекция невозможна.
pub fn range_ts_cond(range: Option<&String>, col: &str) -> String {
    let interval = match range.map(|s| s.as_str()) {
        Some("today") => "1 day",
        Some("week") => "7 days",
        Some("year") => "365 days",
        Some("all") => return "TRUE".to_string(),
        _ => "30 days",
    };
    format!("{col} > now() - interval '{interval}'")
}

fn extract_section(path: &str) -> &str {
    let p = path.trim_start_matches("/api/");
    let p = p
        .trim_start_matches("pub/")
        .trim_start_matches("priv/")
        .trim_start_matches("admin/");
    p.splitn(2, '/').next().unwrap_or("other")
}
