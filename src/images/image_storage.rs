use std::panic::{AssertUnwindSafe, catch_unwind};
use std::sync::Mutex;
use std::path::{Path, PathBuf};
use log::{debug, error};
use once_cell::sync::Lazy;
use crate::errors::Error;
use crate::storage::storage::Storage;
use crate::storage::util::open_local_file;
use crate::conf::CONF;
use crate::util::norm_path;
use img_shrink::EncodeOptionsBuilder;

static IMG_SHRINK_PANIC_HOOK_GUARD: Lazy<Mutex<()>> = Lazy::new(|| Mutex::new(()));

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
	watermark: Option<PathBuf>,
}

pub struct ImageStorageBuilder {
	shrink_mode: Option<ImgShrinkMode>,
	watermark: Option<PathBuf>,
}

impl ImageStorageBuilder {
	pub fn new() -> Self {
		Self { shrink_mode: None, watermark: None }
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

	/// Водяной знак (PNG с альфой) для уменьшенных вариантов.
	/// Мастер `orig` никогда не затрагивается.
	pub fn watermark(mut self, path: impl Into<PathBuf>) -> Self {
		self.watermark = Some(path.into());
		self
	}

	pub fn build(self) -> ImageStorage {
		ImageStorage {
			storage: Storage::new(),
			shrink_mode: self.shrink_mode,
			watermark: self.watermark,
		}
	}
}

impl ImageStorage {
	fn with_caught_img_shrink_panic<T, F>(
		orig_format: &str,
		target_format: &str,
		f: F,
	) -> Result<T, Error>
	where
		F: FnOnce() -> T,
	{
		let guard = IMG_SHRINK_PANIC_HOOK_GUARD
			.lock()
			.map_err(|_| Error::Storage)?;
		let prev_hook = std::panic::take_hook();
		std::panic::set_hook(Box::new(|_| {}));
		let res = catch_unwind(AssertUnwindSafe(f));
		std::panic::set_hook(prev_hook);
		drop(guard);
		res.map_err(|_| {
			error!("img-shrink panic: from={} to={}", orig_format, target_format);
			Error::Storage
		})
	}

	pub fn new() -> Self {
		Self::builder().build()
	}

	pub fn builder() -> ImageStorageBuilder {
		ImageStorageBuilder::new()
	}

	// Хранилище мастеров: приватный контейнер, если задан в конфиге, иначе основной (старое поведение).
	fn orig_storage(private_container_name: &str) -> Storage {
		let name = private_container_name.trim();
		if name.is_empty() {
			Storage::new()
		} else {
			Storage::with_container(name)
		}
	}

	/// Прочитать мастер по стему (без префикса `orig/` и без расширения) из orig-контейнера.
	pub async fn open_orig(&self, rel_path: &Path) -> Result<Vec<u8>, Error> {
		let conf = CONF.read().await;
		let main_format = &conf.main_image_format;
		let path = PathBuf::from(format!("orig/{}.{main_format}", rel_path.display()));
		let orig = Self::orig_storage(&conf.selectel.private_container_name);
		orig.open(&path).await
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
		let orig = Self::orig_storage(&conf.selectel.private_container_name);
		orig.delete(&main_path).await?;
		Ok(())
	}
	pub async fn delete_dir(&self, rel_path: &PathBuf) -> Result<(), Error> {
		let conf = CONF.read().await;
		for size in &conf.image_sizes {
			let path = format!("{}/{}", size.size, rel_path.display());
			self.storage.delete(&path).await?;
		}
		let main_path = PathBuf::from(format!("orig/{}", rel_path.display()));
		let orig = Self::orig_storage(&conf.selectel.private_container_name);
		orig.delete(&main_path).await?;
		Ok(())
	}
	pub async fn save(&self, data: Vec<u8>, path: &PathBuf) -> Result<PathBuf, Error> {
		let orig_format: &str = path.extension().unwrap().to_str().unwrap();

		let conf = CONF.read().await;
		let main_format = &conf.main_image_format;
		let shrink_mode = self
			.shrink_mode
			.unwrap_or_else(|| ImgShrinkMode::from_conf_value(&conf.img_shrink_quality_mode));

		let orig = Self::orig_storage(&conf.selectel.private_container_name);
		let mut path = PathBuf::from(format!("orig/{}", path.display()));
		path.set_extension(main_format);
		let path = orig.get_unique_path(&path).await?;

		let enc_opts = shrink_mode
			.apply(img_shrink::EncodeOptionsBuilder::new().size(&conf.main_image_size))
			.build();
		let main_tmp = Self::with_caught_img_shrink_panic(orig_format, main_format, || {
			img_shrink::encode(&data, orig_format, main_format, enc_opts)
		})?;
		let main_img_data = open_local_file(&main_tmp.path().to_path_buf()).await;
		let mut result_path = orig.save(main_img_data, &path).await?;
		result_path.set_extension("");
		let result_path = result_path.strip_prefix("orig").unwrap().to_path_buf();
		debug!("result path: {}", result_path.display());

		let path = path.strip_prefix("orig").unwrap();
		self.save_variants(&data, orig_format, path, shrink_mode, &conf, false).await?;
		Ok(result_path)
	}

	/// Сгенерировать и залить уменьшенные варианты из исходных байтов.
	/// `overwrite` — писать в точные ключи поверх существующих (перегенерация).
	async fn save_variants(
		&self,
		data: &Vec<u8>,
		orig_format: &str,
		rel_path: &Path,
		shrink_mode: ImgShrinkMode,
		conf: &crate::conf::Conf,
		overwrite: bool,
	) -> Result<(), Error> {
		for format in &conf.image_formats {
			for size in &conf.image_sizes {
				let mut path = PathBuf::from(size.size.clone()).join(rel_path);
				path.set_extension(format);
				let mut enc_builder = img_shrink::EncodeOptionsBuilder::new().size(&size.size);
				enc_builder = shrink_mode.apply(enc_builder);
				if let Some(wm) = &self.watermark {
					enc_builder = enc_builder.watermark(img_shrink::Watermark::new(wm));
				}
				let enc_opts = enc_builder.build();
				let variant_tmp = Self::with_caught_img_shrink_panic(orig_format, format, || {
					img_shrink::encode(data, orig_format, format, enc_opts)
				})?;
				let variant_data = open_local_file(&variant_tmp.path().to_path_buf()).await;
				if overwrite {
					self.storage.save_force_overwrite(variant_data, &path).await?;
				} else {
					self.storage.save(variant_data, &path).await?;
				}
				debug!("variant: {}", path.display());
			}
		}
		Ok(())
	}

	/// Перегенерировать варианты из байтов мастера (в `main_image_format`),
	/// перезаписывая существующие ключи. Мастер `orig` не затрагивается.
	pub async fn regen_variants(&self, data: Vec<u8>, rel_path: &Path) -> Result<(), Error> {
		let conf = CONF.read().await;
		let main_format = conf.main_image_format.clone();
		let shrink_mode = self
			.shrink_mode
			.unwrap_or_else(|| ImgShrinkMode::from_conf_value(&conf.img_shrink_quality_mode));
		self.save_variants(&data, &main_format, rel_path, shrink_mode, &conf, true).await
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
		let orig = Self::orig_storage(&conf.selectel.private_container_name);
		drop(conf);
		let path = norm_path(format!("orig/{}", path.display()));
		let path = PathBuf::from(path.strip_prefix("/").unwrap_or(&path));
		let list: Vec<String> = orig.ls(&path).await;
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
