use crate::http::{ Request, Resp, forbidden };
use crate::request::RequestTools;
use futures_lite::{ Future };

pub mod api;
pub mod sessions;
pub mod hashing;


pub async fn check_su<F, Fut>(req: Request,
    f: F
) -> Resp
where
    F: Fn(Request) -> Fut,
    Fut: Future<Output = Resp>
{
    if !req.is_su() { return forbidden() }
    f(req).await
}
