pub mod api;
pub mod db;
pub mod worker;

use async_channel::{bounded, Receiver, Sender};
use once_cell::sync::OnceCell;

pub struct ViewRecord {
    pub kind: String,
    pub item_id: i64,
    pub ip: String,
}

static SENDER: OnceCell<Sender<ViewRecord>> = OnceCell::new();

pub fn init() -> Receiver<ViewRecord> {
    let (tx, rx) = bounded(2000);
    let _ = SENDER.set(tx);
    rx
}

/// Учесть просмотр материала. `kind` и `item_id` — доменные (их знает проект,
/// не saras): например ("publication", 42). Пишется фоновым воркером пачками.
pub fn record_view(kind: &str, item_id: i64, ip: &str) {
    let Some(tx) = SENDER.get() else { return };
    let _ = tx.try_send(ViewRecord {
        kind: kind.to_string(),
        item_id,
        ip: ip.to_string(),
    });
}
