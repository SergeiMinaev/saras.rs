use lpsql::Lpsql;

use crate::db::get_pool;

pub async fn insert_referrer(host: &str, ip: &str) {
    let pool = get_pool();
    let q = "INSERT INTO referrers (host, ip) VALUES ($1, $2)";
    let _ = Lpsql::query(q).bind(host).bind(ip).exec(&pool).await;
}
