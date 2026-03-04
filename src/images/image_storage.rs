use std::path::PathBuf;
use log::debug;
use crate::errors::Error;
use crate::storage::storage::Storage;
use crate::storage::util::open_local_file;
use crate::conf::CONF;
use crate::util::norm_path;
use img_shrink::EncodeOptionsBuilder;

#[derive(Clone, Copy)]
pub enum ImgShrinkMode {
	Medium,
	High,
}

impl ImgShrinkMode {
	fn from_conf_value(mode: &str) -> Self {
		match mode.trim().to_lowercase().as_str() {
			"high" => Self::High,
			_ => Self::Medium,
		}
	}

	fn apply<'a>(self, builder: EncodeOptionsBuilder<'a>) -> EncodeOptionsBuilder<'a> {
		match self {
			Self::Medium => builder.medium(),
			Self::High => builder.high(),
		}
	}
}

pub struct ImageStorage {
	storage: Storage,
	shrink_mode: Option<ImgShrinkMode>,
}

pub struct ImageStorageBuilder {
	shrink_mode: Option<ImgShrinkMode>,
}

impl ImageStorageBuilder {
	pub fn new() -> Self {
		Self { shrink_mode: None }
	}

	pub fn mode(mut self, mode: ImgShrinkMode) -> Self {
		self.shrink_mode = Some(mode);
		self
	}

	pub fn medium(self) -> Self {
		self.mode(ImgShrinkMode::Medium)
	}

	pub fn high(self) -> Self {
		self.mode(ImgShrinkMode::High)
	}

	pub fn build(self) -> ImageStorage {
		ImageStorage {
			storage: Storage::new(),
			shrink_mode: self.shrink_mode,
		}
	}
}

impl ImageStorage {
	pub fn new() -> Self {
		Self::builder().build()
	}

	pub fn builder() -> ImageStorageBuilder {
		ImageStorageBuilder::new()
	}
	pub async fn delete(&self, rel_path: &PathBuf) -> Result<(), Error> {
		let conf = CONF.read().await;
		for format in &conf.image_formats {
			for size in &conf.image_sizes {
				let path = format!("{}/{}.{format}", size.size, rel_path.display());
				self.storage.delete(&path).await?;
			}
		}
		let main_format = &conf.main_image_format;
		let main_path = PathBuf::from(format!("orig/{}.{main_format}", rel_path.display()));
		self.storage.delete(&main_path).await?;
		Ok(())
	}
	pub async fn delete_dir(&self, rel_path: &PathBuf) -> Result<(), Error> {
		let conf = CONF.read().await;
		for size in &conf.image_sizes {
			let path = format!("{}/{}", size.size, rel_path.display());
			self.storage.delete(&path).await?;
		}
		let main_path = PathBuf::from(format!("orig/{}", rel_path.display()));
		self.storage.delete(&main_path).await?;
		Ok(())
	}
	pub async fn save(&self, data: Vec<u8>, path: &PathBuf) -> Result<PathBuf, Error> {
		let orig_format: &str = path.extension().unwrap().to_str().unwrap();

		let conf = CONF.read().await;
		let main_format = &conf.main_image_format;
		let shrink_mode = self
			.shrink_mode
			.unwrap_or_else(|| ImgShrinkMode::from_conf_value(&conf.img_shrink_quality_mode));

		let mut path = PathBuf::from(format!("orig/{}", path.display()));
		path.set_extension(main_format);
		let path = self.storage.get_unique_path(&path).await?;

		let enc_opts = shrink_mode
			.apply(img_shrink::EncodeOptionsBuilder::new().size(&conf.main_image_size))
			.build();
		let main_tmp = img_shrink::encode(&data, orig_format, main_format, enc_opts);
		let main_img_data = open_local_file(&main_tmp.path().to_path_buf()).await;
		let mut result_path = self.storage.save(main_img_data, &path).await?;
		result_path.set_extension("");
		let result_path = result_path.strip_prefix("orig").unwrap().to_path_buf();
		debug!("result path: {}", result_path.display());

		let path = path.strip_prefix("orig").unwrap();
		for format in &conf.image_formats {
			for size in &conf.image_sizes {
				let mut path = PathBuf::from(size.size.clone()).join(path);
				path.set_extension(format);
				let enc_opts = shrink_mode
					.apply(img_shrink::EncodeOptionsBuilder::new().size(&size.size))
					.build();
				let variant_tmp = img_shrink::encode(&data, orig_format, format, enc_opts);
				let variant_data = open_local_file(&variant_tmp.path().to_path_buf()).await;
				let variant_path = self.storage.save(variant_data, &path).await?;
				debug!("variant: {}", variant_path.display());
			}
		}
		Ok(result_path)
	}
	pub async fn  ls_imgs(&self, path: &PathBuf) -> Vec<String> {
		let conf = CONF.read().await;
		let ext = format!(
			".{}",
			conf.main_image_format
				.trim()
				.trim_start_matches('.')
				.to_lowercase()
		);
		drop(conf);
		let path = norm_path(format!("orig/{}", path.display()));
		let path = PathBuf::from(path.strip_prefix("/").unwrap_or(&path));
		let list: Vec<String> = self.storage.ls(&path).await;
		list.into_iter()
			.filter_map(|s| {
				if s.ends_with('/') {
					Some(s)
				} else if s.to_lowercase().ends_with(&ext) {
					let mut s = s.clone();
					s.truncate(s.len() - ext.len());
					Some(s)
				} else {
					None
				}
			})
			.collect()
	}
}
