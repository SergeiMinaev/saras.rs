use image::{DynamicImage, GenericImageView, imageops::overlay, load_from_memory, RgbaImage, ImageFormat};
use std::io::Cursor;


pub fn overlay_pngs(base_data: Vec<u8>, overlay_data: Vec<u8>) -> Vec<u8> {
    let mut base_img = load_from_memory(&base_data).expect("Failed to load base image").to_rgba8();
    let overlay_img = load_from_memory(&overlay_data).expect("Failed to load overlay image").to_rgba8();
    overlay(&mut base_img, &overlay_img, 0, 0);
    let mut output_data = Vec::new();
    let mut cursor = Cursor::new(&mut output_data);
    base_img.write_to(&mut cursor, ImageFormat::Png).expect("Failed to write image to memory");
    output_data
}
