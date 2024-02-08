use thiserror::Error;
use serde::{ Serialize,Deserialize };
use crate::validation::ValidationErrors;
use crate::storage::errors::StorageError;
use once_cell::sync::Lazy;
use async_lock::RwLock;

/// Use `K` enum to specify message variants.
#[macro_export]
macro_rules! define_msgs {
    ($($key:ident => {$($lang:ident: $value:expr),*}),* $(,)?) => {
        #[derive(Debug, Hash, Eq, PartialEq, Clone, Copy)]
        pub enum K {
            $($key,)*
        }

        impl K {
            pub fn as_str(&self) -> &'static str {
                match self {
                    $(
                        K::$key => stringify!($key),
                    )*
                }
            }
        }

        pub struct Msgs {
            messages: std::collections::HashMap<K, std::collections::HashMap<String, &'static str>>,
        }

        impl Msgs {
            pub fn new() -> Self {
                let mut messages = std::collections::HashMap::new();
                $(
                    let mut lang_map = std::collections::HashMap::new();
                    $(
                        lang_map.insert(stringify!($lang).to_string(), $value);
                    )*
                    messages.insert(K::$key, lang_map);
                )*
                Msgs { messages }
            }

            pub fn msg(&self, key: K, lang: &str) -> String {
                self.messages.get(&key)
                    .and_then(|langs| langs.get(lang))
                    .copied()
                    .unwrap_or(key.as_str())
                    .to_string()
            }

            pub fn dyn_msg(&self, key: K, lang: &str, params: Option<std::collections::HashMap<&str, &str>>) -> String {
                let message = self.messages.get(&key)
                    .and_then(|langs| langs.get(lang))
                    .copied()
                    .unwrap_or("");

                let mut result = message.to_string();
                if let Some(p) = params {
                    for (k, v) in p {
                        result = result.replace(&format!("{{{}}}", k), v);
                    }
                }
                result
            }
        }
    }
}
