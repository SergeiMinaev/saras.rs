pub mod api;
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

fn extract_section(path: &str) -> &str {
    let p = path
        .trim_start_matches("/api/pub/")
        .trim_start_matches("/api/priv/");
    p.splitn(2, '/').next().unwrap_or("other")
}
