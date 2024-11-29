use std::path::PathBuf;
use log::debug;
use crate::http::{ Request, Resp, JsonResp, not_found };
use crate::errors::Error;
use crate::images::image_storage::ImageStorage;
use crate::storage::img_storage::forms::ImgStorageUploadForm;
use crate::storage::img_storage::service;


pub async fn img_storage(req: Request) -> Resp {
	match req.method.as_str() {
		"get" => {
			return ls(req).await
		},
		"post" => {
			return upload(req).await
		},
		"delete" => {
			return delete(req).await
		},
		_ => return not_found(),
	}
}

pub async fn ls(req: Request) -> Resp {
	let path = PathBuf::from(req.query.get("path").unwrap());
	let img_storage = ImageStorage::new();
	let list: Vec<String> = img_storage.ls_imgs(&path).await;
	return JsonResp::ok("").content(&list).to_http()
}

pub async fn upload(req: Request) -> Resp {
    let form: ImgStorageUploadForm = match serde_json::from_str(&req.body_string) {
        Ok(form) => form,
        Err(e) => {
			debug!("upload(): {e}");
			return JsonResp::err("Не удалось загрузить изображение.", &Error::Storage)
			.to_http()
		},
    };
    if let Err(e) = service::upload_img(form).await {
        debug!("upload(): {e}");
        return JsonResp::err("Не удалось загрузить изображение.", &Error::Storage).to_http();
    }
    JsonResp::ok("Изображение загружено.").to_http()
}

pub async fn delete(req: Request) -> Resp {
	let path = req.query.get("path").unwrap();
	debug!("delete: {:?}", req);
	let img_storage = ImageStorage::new();
	if path.ends_with("/") {
		let _ = img_storage.delete_dir(&PathBuf::from(path)).await;
	} else {
		let _ = img_storage.delete(&PathBuf::from(path)).await;
	}
	JsonResp::ok("Файл удалён.").to_http()
}
