use async_channel::Receiver;
use async_std::task;
use std::time::Duration;

use super::db;
use super::RefRecord;

const FLUSH_INTERVAL_SECS: u64 = 5;
const BATCH_SIZE: usize = 500;

pub async fn run(rx: Receiver<RefRecord>) {
    println!("referrers_worker: started");
    loop {
        task::sleep(Duration::from_secs(FLUSH_INTERVAL_SECS)).await;
        let mut batch: Vec<RefRecord> = Vec::with_capacity(BATCH_SIZE);
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
            db::insert_referrer(&rec.host, &rec.ip).await;
        }
    }
}
