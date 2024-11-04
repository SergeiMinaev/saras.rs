use std::path::{Path, PathBuf};
use regex::Regex;
use futures_lite::{AsyncReadExt, StreamExt};
use once_cell::sync::Lazy;
use async_lock::RwLock;
use sha2::{Sha256, Digest};
use crate::conf::CONF;
use crate::memstore::MemStore;


pub static TPL_MEMSTORE: Lazy<RwLock<MemStore>> = Lazy::new(|| {
    RwLock::new(MemStore::new())
});


pub async fn get_index_html(index_path: &str, tpl_dirs: Vec<&str>) -> String {
	let conf = CONF.read().await;
	if conf.is_dev {
		return inc_tpls(index_path, tpl_dirs).await;
	}
	let mut store = TPL_MEMSTORE.write().await;
	match store.get(index_path) {
		Some(html) => return html,
		None => {
			let html = inc_tpls(index_path, tpl_dirs).await;
			store.set(index_path.to_string(), html.clone(), None);
			return html
		}
	};
}

async fn inc_tpls(index_path: &str, tpl_dirs: Vec<&str>) -> String {
	let index_html = async_fs::read_to_string(index_path).await
		.expect(&format!("Unable to open {index_path}"));
	let mut tpls_html = String::new();
	for path in tpl_dirs {
		let dir_entries = async_fs::read_dir(path).await.unwrap()
				.filter_map(Result::ok)
				.filter_map(|d| d.path().to_str().and_then(
					|f| if f.ends_with("/index.html") == false
					&& f.ends_with(".html") { Some(d) } else { None }
				));
		let dir_entries = dir_entries.collect::<Vec<_>>().await;
		for entry in dir_entries {
			let path = entry.path();
			if path.is_file() {
				let tpl_html = async_fs::read_to_string(path).await.unwrap();
				tpls_html.push_str(&tpl_html);
			}
		}
	};
	tpl(index_html.replace("<#inc-templates>", &tpls_html)).await
}

pub async fn tpl(mut text: String) -> String {
	let conf = CONF.read().await;
	let tags = [("DEV", conf.is_dev), ("PROD", !conf.is_dev)];

	for tag in &tags {
		let start_delim = format!("{{% IF {} %}}", tag.0);
		let end_delim = "{% ENDIF %}";
		while let Some(start_idx) = text.find(&start_delim) {
			if let Some(end_idx) = text[start_idx..].find(end_delim) {
				let end_idx = start_idx + end_idx + end_delim.len();
				let inside_content = &text[
					start_idx + start_delim.len()..end_idx - end_delim.len()
				];
				let before_content = &text[..start_idx];
				let after_content = &text[end_idx..];
				if tag.1 {
					text = format!("{before_content}{inside_content}{after_content}");
				} else {
					text = format!("{before_content}{after_content}");
				}
			} else {
				break
			}
		}
	};
    assets_versioning(text).await
}

async fn assets_versioning(html: String) -> String {
	// Example:
	// <a href='{% static /static/img/bg.webp %}'>
	let conf = CONF.read().await;
    let re = Regex::new(r#"\{% static ([^']+?) %\}"#).unwrap();
    let mut result = html.clone();
    for cap in re.captures_iter(&html) {
        let path = trim_quotes(&cap[1]);
		let abs_path = join_paths(&conf.static_dir, &path);
        let hash = calc_hash(abs_path).await;
        let replacement = format!(r#"{}?hash={}"#, path, hash);
        result = result.replace(&cap[0], &replacement);
    }
	result
}


fn trim_quotes(s: &str) -> String {
    let re = Regex::new(r#"^["']|["']$"#).unwrap();
    re.replace_all(s, "").to_string()
}

fn join_paths(base: &str, relative: &str) -> PathBuf {
    let base_path = Path::new(base);
    let mut relative_path = Path::new(relative);
	if relative_path.is_absolute() {
		relative_path = relative_path.strip_prefix("/").unwrap();
	}
	base_path.join(relative_path)
}

async fn calc_hash(path: PathBuf) -> String {
	if !path.is_file() {
		return String::new()
	}
	let mut file = async_fs::File::open(path).await.unwrap();
	let mut hasher = Sha256::new();
	let buf_size = 1024*32;
	let mut buffer = vec![0u8; buf_size];
	loop {
		let n = file.read(&mut buffer).await.unwrap();
		if n == 0 { break; }
		hasher.update(&buffer[..n]);
	}
	let hash = hasher.finalize();
	let hash_hex = format!("{:x}", hash);
	hash_hex[..10].to_string()
}
