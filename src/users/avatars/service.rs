use std::path::PathBuf;
use std::cmp::min;
use std::io::Cursor;
use image::{DynamicImage, ImageEncoder};
use image::codecs::png::PngEncoder;
use image::{Rgb, RgbImage};
use md5;
use palette::{Hsv, Srgb, FromColor};
use log::debug;
use crate::users::avatars::db::AvatarDb;
use crate::errors::Error;
use crate::forms::image_field_form::ImageFieldForm;
use crate::images::image_storage::ImageStorage;
use crate::util::decode_base64;
use crate::db::get_pool;
use crate::users::users::db::UserDb;



/// Update user's avatar. To delete avatar send empty string.
pub async fn update_avatar(user_id: i32, form: &ImageFieldForm) -> Result<(), Error> {
	println!("update_avatar {user_id}");
	let _ = delete_avatar(user_id).await;
	let img_storage = ImageStorage::new();
	let bytes = decode_base64(&form.data_base64.clone().unwrap())?;
	let path = PathBuf::from("users/avatars").join(form.path.clone());
	debug!("ava form path: {}", &form.path.display());
	let path = img_storage.save(bytes, &path).await?;
	debug!("ava cloud path: {}", path.display());
	let pool = get_pool();
	let avatardb = AvatarDb::new(pool);
	avatardb.save(user_id, path.as_path().to_str().unwrap()).await?;
	Ok(())
}


pub async fn delete_avatar(user_id: i32) -> Result<(), Error> {
	let pool = get_pool();
	let avatardb = AvatarDb::new(pool);
	match avatardb.by_user_id(user_id).await {
		Err(()) => Error::Database,
		Ok(existing_avatar) => {
			let img_storage = ImageStorage::new();
			match img_storage.delete(&PathBuf::from(existing_avatar.path)).await {
				Ok(_) => {
					avatardb.delete(user_id).await;
					return Ok(())
				},
				Err(e) => return Err(e)
			}
		}
	};
	Ok(())
}


pub async fn delete_default_avatar(user_id: i32) -> Result<(), Error> {
	let pool = get_pool();
	let avatardb = AvatarDb::new(pool);
	match avatardb.default_by_user_id(user_id).await {
		Err(()) => Error::Database,
		Ok(existing_avatar) => {
			let img_storage = ImageStorage::new();
			match img_storage.delete(&PathBuf::from(existing_avatar.path)).await {
				Ok(_) => {
					avatardb.delete_default(user_id).await;
					return Ok(())
				},
				Err(e) => return Err(e)
			}
		}
	};
	Ok(())
}


pub async fn update_default_avatar(user_id: i32) -> Result<(), Error> {
	delete_default_avatar(user_id).await;
	let pool = get_pool();
	let userdb = UserDb::new(pool.clone());
	let user = userdb.by_id(user_id).await.unwrap();
	if user.name.is_none() {
		return Ok(())
	}
	let name = user.name.unwrap();
	let ava = generate_avatar(&name);
	let img_storage = ImageStorage::new();
	let path = PathBuf::from("users/avatars").join(format!("{name}.png"));
	let bytes = rgb_image_to_bytes(&ava);
	let path = img_storage.save(bytes, &path).await?;
	let avatardb = AvatarDb::new(pool.clone());
	avatardb.save_default(user_id, path.as_path().to_str().unwrap()).await?;
	Ok(())
}


pub fn generate_avatar(name: &str) -> RgbImage {
	_generate_avatar(name, 8, 20, 0.5, 0.4)
}


fn _generate_avatar(
    name: &str,
    grid_size: u32,
    scale: u32,
    min_saturation: f32,
    min_brightness: f32,
) -> RgbImage {
    let hash = md5::compute(name);
    let hash_hex = format!("{:032x}", hash);

	// color from hash
    let hue = u8::from_str_radix(&hash_hex[0..2], 16).unwrap() as f32 / 255.0;
    let mut saturation = u8::from_str_radix(&hash_hex[2..4], 16).unwrap() as f32 / 255.0;
    if saturation < min_saturation {
        saturation = min_saturation;
    }
    let mut brightness = u8::from_str_radix(&hash_hex[4..6], 16).unwrap() as f32 / 255.0;
    if brightness < min_brightness {
        brightness = min_brightness;
    }
    let hsv = Hsv::new(hue * 360.0, saturation, brightness);
	let rgb: Srgb = Srgb::from_color(hsv);
    let r = (rgb.red * 255.0).round() as u8;
    let g = (rgb.green * 255.0).round() as u8;
    let b = (rgb.blue * 255.0).round() as u8;
    let color = Rgb([r, g, b]);
    let bg_color = Rgb([240, 240, 240]);

    let img_size = grid_size * scale;
    let mut image = RgbImage::from_pixel(img_size, img_size, bg_color);

    let half = (grid_size + 1) / 2;

    for y in 0..grid_size {
        let mut row: Vec<bool> = Vec::with_capacity(half as usize);
        for x in 0..half {
            let index = (y * half + x) as usize;
            let digit_char = hash_hex.chars().nth(index).unwrap();
            let digit = digit_char.to_digit(16).unwrap();
            let cell_on = digit % 2 == 0;
            row.push(cell_on);
        }

		// mirror
        let mut full_row = row.clone();
        if grid_size % 2 == 0 {
            let mut rev = row.clone();
            rev.reverse();
            full_row.extend(rev);
        } else {
            let mut rev = row[..row.len()-1].to_vec();
            rev.reverse();
            full_row.extend(rev);
        }
		// draw cells
        for (x, &cell) in full_row.iter().enumerate() {
            if cell {
                let start_x = (x as u32) * scale;
                let start_y = y * scale;
                for dy in 0..scale {
                    for dx in 0..scale {
                        image.put_pixel(start_x + dx, start_y + dy, color);
                    }
                }
            }
        }
    }

    image
}


fn rgb_image_to_bytes(img: &RgbImage) -> Vec<u8> {
    let mut buf = Vec::new();
    let dynamic_img = DynamicImage::ImageRgb8(img.clone()); // Преобразование в DynamicImage

    PngEncoder::new(&mut buf).write_image(
        dynamic_img.as_bytes(),
        dynamic_img.width(),
        dynamic_img.height(),
        dynamic_img.color().into(),
    ).unwrap();

    buf
}
