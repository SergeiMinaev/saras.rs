use base64::{ Engine as _, engine::{ general_purpose } };
use schemars::schema::{ Schema, SchemaObject, Metadata };
use schemars::{gen::SchemaGenerator, JsonSchema};
use serde::{ Serialize,Deserialize };
use crate::storage;
use std::path::PathBuf;
use crate::conf::CONF;
use crate::util::slugify;




#[derive(Serialize, Deserialize, Debug)]
pub struct ImageStorage {
  path: String,
}

impl ImageStorage {
	pub fn new(path: String) -> Self {
		ImageStorage { path: path }
	}
	pub async fn save_from_base64(data: &str, dir: &str) -> PathBuf {
		let mut split = data.split(",");
		let fname = split.nth(0).unwrap_or_default();
		let path = format!("{dir}/{fname}");
		let content = split.nth(1).unwrap_or_default();
		let bytes = general_purpose::STANDARD.decode(content).unwrap();
		let path = ImageStorage::save(&bytes, &path).await;
		path
	}
	fn clean_path(path: &str) -> PathBuf {
		PathBuf::from(slugify(path))
	}
	pub async fn save(content: &Vec<u8>, path: &str) -> PathBuf {
		let path = ImageStorage::clean_path(path);
		let conf = CONF.read().await;
		let format: &str = path.extension().unwrap().to_str().unwrap();
		let png_path = img_shrink::make_png(content, format);
		let main_format = &conf.main_image_format;
		let main_size = "1900x1900";
		let main_img = img_shrink::make_version(&png_path, main_format, main_size, false);

		let main_version_content = storage::open_local_file(&main_img).await;
		let mut main_cloud_path  = PathBuf::from(format!("orig/{}", path.display()));
		main_cloud_path.set_extension(main_format);
		let cloud_path = storage::save_file(&main_version_content, &main_cloud_path).await;
		// File name may be modified if one is already exist.
		let cloud_filename = cloud_path.file_name().unwrap();
		let mut path = path.clone();
		path.set_file_name(cloud_filename);

		for format in &conf.image_formats {
			for variant in &conf.image_sizes {
				let variant_path = img_shrink::make_version(&png_path, format, &variant.size, variant.crop);
				let variant_content = storage::open_local_file(&variant_path).await;
				let mut variant_cloud_path = PathBuf::from(format!("{}/{}", variant.size, path.display()));
				variant_cloud_path.set_extension(format);
				let _ = storage::save_file(&variant_content, &variant_cloud_path).await;
			}
		}
		path
	}
	pub async fn delete(path: &str) {
		let conf = CONF.read().await;
		for format in &conf.image_formats {
			for size in &conf.image_sizes {
				let xpath = PathBuf::from(format!("{}/{path}.{format}", size.size));
				storage::delete_file(&xpath).await;
			}
		}
		let orig_path = PathBuf::from(format!("orig/{path}.{}", conf.main_image_format));
		storage::delete_file(&orig_path).await;
	}
}
