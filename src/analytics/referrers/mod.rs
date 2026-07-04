pub mod api;
pub mod db;
pub mod worker;

use async_channel::{bounded, Receiver, Sender};
use once_cell::sync::OnceCell;

use crate::http::Request;
use crate::request::client_ip;

pub struct RefRecord {
    pub host: String,
    pub ip: String,
}

static SENDER: OnceCell<Sender<RefRecord>> = OnceCell::new();

pub fn init() -> Receiver<RefRecord> {
    let (tx, rx) = bounded(1000);
    let _ = SENDER.set(tx);
    rx
}

pub fn track(host: String, ip: String) {
    let Some(tx) = SENDER.get() else { return };
    let _ = tx.try_send(RefRecord { host, ip });
}

/// Учитывает внешний источник перехода из заголовка `Referer` запроса-документа.
/// Свой домен (по `req.host`, без `www.`) и прямые заходы (реферера нет)
/// пропускает. Generic — зовётся из index-хендлера любого проекта.
pub fn track_from_request(req: &Request) {
    let Some(referer) = req.headers.get("referer") else {
        return;
    };
    let Some(host) = host_from_referrer(referer) else {
        return;
    };
    let own = req.host.strip_prefix("www.").unwrap_or(&req.host);
    let ref_host = host.strip_prefix("www.").unwrap_or(&host);
    if ref_host == own {
        return;
    }
    track(host, client_ip(req));
}

/// Извлекает хост из значения `document.referrer` / заголовка `Referer`.
///
/// Возвращает None для пустой строки, строки без хоста или явно битого значения.
/// Путь/query отбрасываются намеренно - храним только источник (хост), не URL.
pub fn host_from_referrer(referrer: &str) -> Option<String> {
    let s = referrer.trim();
    if s.is_empty() {
        return None;
    }
    // Отбрасываем схему (`https://`), затем берём хост до первого '/', '?' или '#'.
    let after_scheme = s.split_once("://").map(|(_, r)| r).unwrap_or(s);
    let host = after_scheme.split(['/', '?', '#']).next().unwrap_or("");
    // Отбрасываем userinfo@ и :port.
    let host = host.rsplit('@').next().unwrap_or(host);
    let host = host.split(':').next().unwrap_or(host);
    let host = host.trim().to_lowercase();

    if host.is_empty() || host.len() > 255 || !host.contains('.') {
        return None;
    }
    if !host
        .chars()
        .all(|c| c.is_ascii_alphanumeric() || c == '.' || c == '-')
    {
        return None;
    }
    Some(host)
}
