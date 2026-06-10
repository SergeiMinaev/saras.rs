pub mod api;
pub mod db;
pub mod worker;

use async_channel::{bounded, Receiver, Sender};
use once_cell::sync::OnceCell;

pub struct VisitRecord {
    pub ip: String,
    pub session_id: String,
    pub is_404: bool,
}

static SENDER: OnceCell<Sender<VisitRecord>> = OnceCell::new();

pub fn init() -> Receiver<VisitRecord> {
    let (tx, rx) = bounded(1000);
    let _ = SENDER.set(tx);
    rx
}

pub fn track(ip: String, session_id: String, is_404: bool) {
    let Some(tx) = SENDER.get() else { return };
    let _ = tx.try_send(VisitRecord { ip, session_id, is_404 });
}

pub fn ip_to_prefix(ip: &str) -> String {
    let parts: Vec<&str> = ip.split('.').collect();
    if parts.len() >= 3 {
        parts[..3].join(".")
    } else {
        ip.to_string()
    }
}

pub fn dedup_key(s: &str) -> String {
    let digest = md5::compute(s.as_bytes());
    format!("{:x}", digest)[..16].to_string()
}
