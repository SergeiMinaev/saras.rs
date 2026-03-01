use log::debug;
use serde_json::json;
use crate::errors::Error;
use crate::http::{not_found, JsonResp, Request, Resp};
use crate::storage::file_storage::forms::FileStorageUploadForm;
use crate::storage::file_storage::service;
use crate::storage::storage::Storage;

pub async fn file_storage(req: Request) -> Resp {
	match req.method.as_str() {
		"get" => ls(req).await,
		"post" => upload(req).await,
		"delete" => delete(req).await,
		_ => not_found(),
	}
}

pub async fn ls(req: Request) -> Resp {
	let raw = req
		.query
		.get("path")
		.cloned()
		.unwrap_or("/".to_string());
	let path = service::scoped_storage_path(&raw);
	let storage = Storage::new();
	let list: Vec<String> = storage.ls(&path).await;
	JsonResp::ok("").content(&list).to_http()
}

pub async fn upload(req: Request) -> Resp {
	let form: FileStorageUploadForm = match serde_json::from_str(&req.body_string) {
		Ok(form) => form,
		Err(e) => {
			debug!("file_storage upload(): {e}");
			return JsonResp::err("Не удалось загрузить файл.", &Error::Storage).to_http();
		}
	};
	let path = match service::upload_file(form).await {
		Ok(path) => path,
		Err(e) => {
			debug!("file_storage upload(): {e}");
			return JsonResp::err("Не удалось загрузить файл.", &Error::Storage).to_http();
		}
	};
	JsonResp::ok("Файл загружен.")
		.content(&json!({
			"path": path.display().to_string(),
		}))
		.to_http()
}

pub async fn delete(req: Request) -> Resp {
	let raw = req.query.get("path").unwrap();
	let is_dir = raw.ends_with('/');
	let path = service::scoped_storage_path(raw);
	match service::delete_file(&path, is_dir).await {
		Ok(_) => JsonResp::ok("Файл удалён.").to_http(),
		Err(e) => {
			debug!("file_storage delete(): {e}");
			JsonResp::err("Не удалось удалить файл.", &Error::Storage).to_http()
		}
	}
}
