use std::sync::Arc;
use lpsql::pool::ConnectionPool;
use once_cell::sync::Lazy;


static CONNECTION_POOL: Lazy<Arc<ConnectionPool>> = Lazy::new(|| {
    Arc::new(ConnectionPool::new(5, None, None))
});

pub fn get_pool() -> Arc<ConnectionPool> {
    CONNECTION_POOL.clone()
}
