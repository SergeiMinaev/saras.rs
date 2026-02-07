use serde_json::{json, Value};
use crate::legal_docs::docs;

pub fn legal_docs_schema() -> Value {
    json!({
        "label": "Юр. доккументы",
        "endpoint": "legal_docs",
        "models": [
            docs::schemas::admin_schemas(),
        ],
    })
}
