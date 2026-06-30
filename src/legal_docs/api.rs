use crate::conf::CONF;
use crate::errors::Error;
use crate::http::{not_found, JsonResp, Request, Resp};
use crate::legal_docs::consents;
use crate::legal_docs::files;
use crate::request::RequestTools;
use serde_json::{json, Value};

/// Отдаёт юр-документ из файла (`<legal_dir>/<key>.html` + заголовок из manifest).
/// Поле `id` - для совместимости с фронтом (ContentBlockPage проверяет `data.id`).
pub async fn doc_by_key(req: Request) -> Resp {
    let Some(key) = req.route.get("key").cloned() else {
        return not_found();
    };
    let (Some(title), Some(html)) = (files::doc_title(&key).await, files::doc_html(&key).await) else {
        return not_found();
    };
    let r = json!({
        "id": key,
        "key": key,
        "title": title,
        "html": html,
    });
    JsonResp::ok("").content(&r).to_http()
}

const ALLOWED_SOURCES: [&str; 2] = ["web", "app"];

/// Приём согласия на обработку ПДн. Личность - из сессии, версия документа - из конфига.
/// `source` различает источник явным полем ("web" | "app"). Общий хендлер для всех приложений.
pub async fn accept(req: Request) -> Resp {
    if req.method.to_lowercase() != "post" {
        return not_found();
    }

    let Some(user) = req.get_user().await else {
        return JsonResp::err("Требуется авторизация.", &Error::Auth)
            .code(401)
            .to_http();
    };

    let body: Value = serde_json::from_str(&req.body_string).unwrap_or(Value::Null);
    let source = body.get("source").and_then(|v| v.as_str()).unwrap_or("");
    if !ALLOWED_SOURCES.contains(&source) {
        return JsonResp::err("Некорректный источник согласия.", &Error::Validation).to_http();
    }

    let (key, version) = {
        let conf = CONF.read().await;
        (
            conf.legal_docs.consent_key.clone(),
            conf.legal_docs.consent_version.clone(),
        )
    };

    let ip = req
        .headers
        .get("x-real-ip")
        .cloned()
        .or_else(|| {
            req.headers
                .get("x-forwarded-for")
                .and_then(|v| v.split(',').next().map(|s| s.trim().to_string()))
        })
        .filter(|v| !v.is_empty());
    let user_agent = req
        .headers
        .get("user-agent")
        .cloned()
        .filter(|v| !v.is_empty());

    // create_consent идемпотентен (on conflict do nothing): повторное согласие той же
    // версии вернёт false, но это не ошибка - пользователь уже согласен.
    consents::create_consent(user.id as i32, &key, &version, source, ip, user_agent).await;

    JsonResp::ok("Согласие сохранено.").to_http()
}
