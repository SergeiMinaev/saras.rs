use async_channel::Receiver;
use async_std::task;
use std::time::Duration;

use super::db;
use super::ViewRecord;

const FLUSH_INTERVAL_SECS: u64 = 5;
const BATCH_SIZE: usize = 500;

pub async fn run(rx: Receiver<ViewRecord>) {
    println!("content_views_worker: started");
    loop {
        task::sleep(Duration::from_secs(FLUSH_INTERVAL_SECS)).await;
        let mut batch: Vec<ViewRecord> = Vec::with_capacity(BATCH_SIZE);
        while let Ok(rec) = rx.try_recv() {
            batch.push(rec);
            if batch.len() >= BATCH_SIZE {
                break;
            }
        }
        if batch.is_empty() {
            continue;
        }
        for rec in &batch {
            db::insert_view(&rec.kind, rec.item_id, &rec.ip).await;
        }
    }
}
