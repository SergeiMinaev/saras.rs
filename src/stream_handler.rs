use async_net::unix::{UnixStream};
use futures_lite::{Future, AsyncReadExt, AsyncWriteExt};
use crate::http::{Resp};
use miarh_saras_http::Request;




pub struct StreamHandler {
	pub stream: UnixStream,
}

impl StreamHandler {
	pub async fn process<Fut>(
		&mut self, url_dispatcher: impl FnOnce(Request) -> Fut)
	where
		Fut: Future<Output = Resp>,
	{
		let mut buf: Vec<u8> = vec![];
		self.stream.read_to_end(&mut buf).await;
		let req: Request = bincode::deserialize(&buf).unwrap();
		let resp = (url_dispatcher)(req).await;
		self.write_resp(&resp).await;
	}
	pub fn new(stream: UnixStream) -> Self {
		Self { stream: stream }
	}
	pub async fn write_resp(&mut self, resp: &Resp) {
		let _ = self.stream.write(resp.get_resp().as_bytes()).await;
	}
}
