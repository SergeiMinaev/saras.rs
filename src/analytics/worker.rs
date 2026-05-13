use async_channel::Receiver;
use async_std::task;
use lpsql::Lpsql;
use std::time::Duration;

use crate::db::get_pool;
use super::HitRecord;

const FLUSH_INTERVAL_SECS: u64 = 5;
const BATCH_SIZE: usize = 500;

pub async fn run(rx: Receiver<HitRecord>) {
    println!("analytics_worker: started");
    loop {
        task::sleep(Duration::from_secs(FLUSH_INTERVAL_SECS)).await;
        let mut batch: Vec<HitRecord> = Vec::with_capacity(BATCH_SIZE);
        while let Ok(hit) = rx.try_recv() {
            batch.push(hit);
            if batch.len() >= BATCH_SIZE {
                break;
            }
        }
        if batch.is_empty() {
            continue;
        }
        flush(&batch).await;
    }
}

async fn flush(batch: &[HitRecord]) {
    let pool = get_pool();
    let q = "INSERT INTO analytics_hits (ip, path, section) VALUES ($1, $2, $3)";
    for hit in batch {
        let _ = Lpsql::query(q)
            .bind(&hit.ip)
            .bind(&hit.path)
            .bind(&hit.section)
            .exec(&pool)
            .await;
    }
}
