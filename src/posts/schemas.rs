use serde_json::{json, Value};
use crate::posts::posts; 


pub fn posts_schema() -> Value {
    json!({
      "label": "Публикации",
      "endpoint": "posts",
      "models": [
        posts::schemas::admin_schemas(),
      ],
    })
}

