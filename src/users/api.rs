use crate::http::{ Request, Resp, JsonResp };
use crate::users::schemas; 


pub async fn admin_schema(_req: Request) -> Resp {
    return JsonResp::ok("").content(schemas::users_schema()).to_http()
}
