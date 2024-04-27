use base64::Engine;
use rex::font::backend::ttf_parser::TtfMathFont;



pub fn load_font<'a>(file : &'a [u8]) -> rex::font::backend::ttf_parser::TtfMathFont<'a> {
    let font = ttf_parser::Face::parse(file, 0).unwrap();
    TtfMathFont::new(font).unwrap()
}



pub fn small_ascii_repr(input : &str) -> String {
    let mut to_return = String::new();
    for c in input.chars().take(20) {
        match c {
            c if 
                   c.is_ascii_alphanumeric() 
                || " +:-<>=()_^?!&{}".contains(c) // some allowable innocuous characters
            => {
                to_return.push(c);
            },
            _ => to_return.push(' '),
        };
    }
    to_return
}


pub const HASH_SIZE : usize = 8;
pub fn simple_hash(input : &[u8]) -> [u8; HASH_SIZE] {
    let mut to_return = [0; HASH_SIZE];
    for chunk in input.chunks(HASH_SIZE) {
        for (value, character) in Iterator::zip(to_return.iter_mut(), chunk.into_iter()) {
            *value = u8::wrapping_add(*value, *character);
        }
    }
    to_return
}


pub fn equation_to_filepath(equation: &str, description : &str) -> String {
    let mut bytes = description.as_bytes().to_vec();
    bytes.extend_from_slice(equation.as_bytes());

    format!(
        "{}-{}.png",
        &small_ascii_repr(equation),
        base64::engine::general_purpose::URL_SAFE_NO_PAD.encode(simple_hash(&bytes)),
    )
}