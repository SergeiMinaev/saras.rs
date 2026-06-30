use crate::conf::CONF;
use serde_json::Value;

/// Каталог с юр-документами: `<static_dir>/<legal_docs.dir>` (канон в git каждого приложения).
async fn legal_dir() -> String {
    let conf = CONF.read().await;
    format!(
        "{}/{}",
        conf.static_dir.trim_end_matches('/'),
        conf.legal_docs.dir.trim_matches('/')
    )
}

/// Заголовок документа из `manifest.json` по ключу.
pub async fn doc_title(key: &str) -> Option<String> {
    let dir = legal_dir().await;
    let raw = std::fs::read_to_string(format!("{dir}/manifest.json")).ok()?;
    let manifest: Value = serde_json::from_str(&raw).ok()?;
    manifest
        .get(key)?
        .get("title")?
        .as_str()
        .map(|s| s.to_string())
}

/// HTML-тело документа по ключу. Ключ из URL - защищаемся от выхода за каталог.
pub async fn doc_html(key: &str) -> Option<String> {
    if key.contains('/') || key.contains("..") {
        return None;
    }
    let dir = legal_dir().await;
    std::fs::read_to_string(format!("{dir}/{key}.html")).ok()
}
