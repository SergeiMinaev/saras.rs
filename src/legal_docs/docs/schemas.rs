use crate::serde_json::{ json, Value };
use crate::legal_docs::docs::models::Doc;
use crate::schema::{ model_meta };
use crate::schema::schemars::{ schema_for };

pub fn doc_view_schema() -> Value {
    json!(schema_for!(Doc))
}

pub fn admin_schemas() -> Value {
    json!({
        "meta": model_meta::<Doc>(),
        "admin": {
            "list_fields": ["id", "key", "title", "version", "is_active"],
            "item_fields": ["id", "key", "title", "html", "version", "is_active"],
        },
        "schemas": {
            "view": doc_view_schema(),
        },
    })
}
