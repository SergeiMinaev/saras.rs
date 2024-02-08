use std::path::PathBuf;
use std::fs::File;
use std::io::prelude::*;


pub async fn open_local_file(path: &PathBuf) -> Vec<u8> {
	let mut file = File::open(path).unwrap();
	let mut buffer: Vec<u8> = vec![];
	file.read_to_end(&mut buffer).unwrap();
	buffer
}
