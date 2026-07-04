use serde_json::{json, Value};

use crate::http::{JsonResp, Request, Resp};

/// Generic-агрегатор просмотров без заголовков материалов.
///
/// Проекту, которому нужны заголовки/ссылки, следует вместо этого звать
/// `content_views::db::top_views` и джойнить item_id на свои таблицы —
/// см. пример в zenux (`/api/admin/analytics/materials`).
pub async fn content_views(req: Request) -> Resp {
    let rows = super::db::top_views(req.query.get("range"), 500).await;
    let items: Vec<Value> = rows
        .iter()
        .map(|r| {
            json!({
                "kind": r.kind,
                "item_id": r.item_id,
                "count": r.count,
                "uniq": r.uniq,
            })
        })
        .collect();
    JsonResp::ok("").content(&json!({ "items": items })).to_http()
}
