use futures_lite::{Future, FutureExt};
use urlmatch::urlmatch;
use crate::listener;
use crate::http;
use crate::http::{Resp};
use miarh_saras_http::Request;


struct Path<Fut>
where
    Fut: Future<Output = Resp>,
{
    p: &'static str,
    f: fn(Request) -> Fut,
}

async fn url_dispatcher(mut req: Request) -> Resp {
    let url = req.path.to_string();
    let routes = vec![
        Path {p: &"/profile", f: |args| profile(args).boxed()},
        Path {p: &"/catalogue/:ctg/:id", f: |args| catalogue(args).boxed()},
        Path {p: &"/json", f: |args| get_json(args).boxed()},
    ];
    for route in routes.iter() {
        let r = urlmatch(&url, route.p);
        req.route = r.keys;
        if r.is_matched {
            return (route.f)(req).await;
        }
    }
    http::text_resp(404, "Not found".to_string())
}

async fn profile(req: Request) -> Resp {
    let text = format!("profile(), route: {:?}", req.route);
    http::text_resp(200, text)
}
async fn catalogue(req: Request) -> Resp {
    let text = format!("catalogue(), route: {:?}", req.route);
    http::text_resp(200, text)
}
async fn get_json(_req: Request) -> Resp {
    let json = format!(r#"
        {{
            "name": "Adam",
            "age": "{}"
        }}
    "#, i64::MIN);
    http::json_resp(200, json)
}

pub async fn run() {
    listener::run(url_dispatcher).await;
}
