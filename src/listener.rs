use std::fs;
use futures_lite::Future;
use async_net::unix::{UnixListener, UnixStream};
use qpidfile::Pidfile;
use crate::stream_handler::{StreamHandler};
use crate::conf::CONF;
use crate::spawn::spawn;
use crate::http::{Resp};
use miarh_saras_http::Request;


pub async fn run<Fut: 'static>(
    url_dispatcher: impl FnOnce(Request) -> Fut + std::marker::Send + 'static + Copy
)
where
    Fut: Future<Output = Resp> + std::marker::Send,
{
    let _pidfile = match Pidfile::new("saras.pid") {
        Ok(v) => v,
        Err(e) => panic!("Unable to create pidfile: {e}")
    };
    let conf = CONF.read().await;
    match fs::remove_file(&conf.socket_path) {
        Ok(_) => {},
        Err(e) => println!("{} ({})", e, &conf.socket_path),
    };
    let listener = UnixListener::bind(&conf.socket_path).unwrap();
    loop {
        let (stream, _peer_addr) = listener.accept().await.unwrap();
        spawn(handle_stream(stream, url_dispatcher)).detach();
    }
}

async fn handle_stream<Fut>(
    stream: UnixStream, url_dispatcher: impl FnOnce(Request) -> Fut)
where
    Fut: Future<Output = Resp>,
{
    let mut handler = StreamHandler::new(stream);
    handler.process(url_dispatcher).await;
}
