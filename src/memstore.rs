use std::str;
use std::any::type_name;
use serde_json::{json, Value};
use crate::schema::schemars::{ JsonSchema };
use crate::models::BaseModel;
use std::collections::HashMap;
use once_cell::sync::Lazy;
use async_lock::RwLock;
use chrono::{ DateTime, Utc, Duration };


pub static MEMSTORE: Lazy<RwLock<MemStore>> = Lazy::new(|| {
    RwLock::new(MemStore::new())
});


pub struct MemStoreItem {
  value: String,
  expiry: Option<DateTime<Utc>>,
}

pub struct MemStore {
  pub data: HashMap<String, String>,
  pub timed_data: HashMap<String, MemStoreItem>,
}

impl MemStore {
  pub fn new() -> Self {
      let s = MemStore { data: HashMap::new(), timed_data: HashMap::new() };
      return s;
  }
  pub fn set(&mut self, key: String, value: String, expiry: Option<Duration>) {
    let expiry = match expiry {
      Some(duration) => Some(Utc::now() + duration),
      None => None,
    };
    let item = MemStoreItem { value, expiry };
    self.timed_data.insert(key, item);
  }
  pub fn del(&mut self, key: &str) {
    self.timed_data.remove(key);
  }
  pub fn get(&mut self, key: &str) -> Option<String> {
    match self.timed_data.get(key) {
      Some(item) => {
        if item.expiry == None {
          return Some(item.value.clone())
        };
        if item.expiry > Some(Utc::now()) {
          return Some(item.value.clone())
        } else {
          println!("EXPIRED");
          self.del(key);
          return None
        }
      },
      None => None
    }
  }
}


pub fn model_meta<T: JsonSchema + BaseModel >() -> Value {
  let mut split = type_name::<T>().split("::");
  let ctg = split.nth(1).unwrap();
  let model = split.nth(2).unwrap();
  let endpoint = format!("{ctg}/{}", model.to_lowercase() + "s");
  return json!({
    "endpoint": endpoint, "model_name": model,
    "name": T::NAME, "name_plural": T::NAME_PLURAL
  });
}

pub fn to_json<T: JsonSchema + BaseModel  + serde::Serialize>(m: T) -> Value {
  json!(m)
}
