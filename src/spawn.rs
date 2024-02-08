use futures_lite::future;
use std::thread;
use once_cell::sync::Lazy;
use async_executor::{Executor, Task};
use std::future::Future;
use std::panic::catch_unwind;


// https://github.com/smol-rs/smol/blob/master/src/spawn.rs
pub fn spawn<T: Send + 'static>(future: impl Future<Output = T> + Send + 'static) -> Task<T> {
    static GLOBAL: Lazy<Executor<'_>> = Lazy::new(|| {
        thread::Builder::new()
            .name(format!("smol-1"))
            .spawn(|| loop {
                catch_unwind(|| future::block_on(
                        GLOBAL.run(future::pending::<()>())
                )).ok();
            })
            .expect("cannot spawn executor thread");
        Executor::new()
    });

    GLOBAL.spawn(future)
}
