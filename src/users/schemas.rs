use serde_json::{json, Value};
use crate::users::users; 


pub fn users_schema() -> Value {
    json!({
      "label": "Пользователи",
      "endpoint": "users",
      "models": [
        users::schemas::admin_schemas(),
      ],
    })
}

