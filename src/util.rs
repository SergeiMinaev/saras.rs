use std::collections::HashMap;
use std::path::Path;
use base64::{ Engine as _, engine::{ general_purpose } };
use crate::errors::Error;


pub fn norm_path(path: String) -> String {
    let mut result = String::new();
    let mut prev_char = '/';
    
    for c in path.chars() {
        if c == '/' && prev_char == '/' {
            continue;
        }
        result.push(c);
        prev_char = c;
    }
    
    if result.ends_with('/') {
        result.pop();
    }
    
    if !result.starts_with('/') {
        result.insert(0, '/');
    }
    
    result
}

fn create_translit_map() -> HashMap<char, &'static str> {
	let pairs = [
		('А', "A"), ('Б', "B"), ('В', "V"), ('Г', "G"), ('Д', "D"), ('Е', "E"), ('Ё', "E"),
		('Ж', "ZH"), ('З', "Z"), ('И', "I"), ('Й', "Y"), ('К', "K"), ('Л', "L"), ('М', "M"),
		('Н', "N"), ('О', "O"), ('П', "P"), ('Р', "R"), ('С', "S"), ('Т', "T"), ('У', "U"),
		('Ф', "F"), ('Х', "KH"), ('Ц', "TS"), ('Ч', "CH"), ('Ш', "SH"), ('Щ', "SHCH"), ('Ъ', ""),
		('Ы', "Y"), ('Ь', ""), ('Э', "E"), ('Ю', "YU"), ('Я', "YA"),

		('а', "a"), ('б', "b"), ('в', "v"), ('г', "g"), ('д', "d"), ('е', "e"), ('ё', "e"),
		('ж', "zh"), ('з', "z"), ('и', "i"), ('й', "y"), ('к', "k"), ('л', "l"), ('м', "m"),
		('н', "n"), ('о', "o"), ('п', "p"), ('р', "r"), ('с', "s"), ('т', "t"), ('у', "u"),
		('ф', "f"), ('х', "kh"), ('ц', "ts"), ('ч', "ch"), ('ш', "sh"), ('щ', "shch"), ('ъ', ""),
		('ы', "y"), ('ь', ""), ('э', "e"), ('ю', "yu"), ('я', "ya"),
	];
	pairs.iter().cloned().collect()
}

pub fn _translit<T: AsRef<Path>>(input: T, slugify: bool) -> String {
	let map = create_translit_map();
	let allowed_specials = "-/.";
	let mut output = String::new();
    for c in input.as_ref().to_str().unwrap().chars() {
		if slugify && c == ' ' {
			output.push_str("-");
			continue;
		}
        if let Some(transliterated) = map.get(&c) {
            output.push_str(transliterated);
        } else {
			if c.is_ascii_alphabetic() || c.is_alphanumeric() || allowed_specials.contains(c) {
				output.push(c);
			}
        }
    }
    output
}

pub fn slugify<T: AsRef<Path>>(input: T) -> String {
	_translit(input, true)
}

pub fn decode_base64(content: &str) -> Result<Vec<u8>, Error> {
	let mut split = content.split(",");
	let content = split.nth(1).unwrap_or_default();
	general_purpose::STANDARD.decode(content).map_err(|_| Error::Decode)
}
