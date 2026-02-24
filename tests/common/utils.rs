use base64::Engine;
use rex::font::backend::ttf_parser::TtfMathFont;
use sha2::Digest;



pub fn load_font<'a>(file : &'a [u8]) -> rex::font::backend::ttf_parser::TtfMathFont<'a> {
    let font = ttf_parser::Face::parse(file, 0).unwrap();
    TtfMathFont::new(font).unwrap()
}



pub fn small_ascii_repr(input : &str) -> String {
    let mut to_return = String::new();
    for c in input.chars().take(20) {
        match c {
            c if  c.is_ascii_alphanumeric() || "_-".contains(c) // some allowable innocuous characters
            => {
                to_return.push(c.to_ascii_lowercase());
            },
            _ => to_return.push('_'),
        };
    }
    to_return
}


pub fn equation_to_filepath(equation: &str, description : &str) -> String {
    let mut bytes = description.as_bytes().to_vec();
    bytes.extend_from_slice(equation.as_bytes());

    let hash = sha2::Sha256::digest(&bytes);
    let filename = format!(
        "{}-{}.png",
        &small_ascii_repr(equation),
        base64::engine::general_purpose::URL_SAFE_NO_PAD.encode(hash),
    );
    
    // Normally, the 'assert' below should pass. 
    // small_ascii_repr returns at most 20 bytes
    // sha256 is at most 32 bytes
    // base64 is at must 4 / 3 x more than original
    // So max <= 20 + 1 + 4 / 3 x 32 + 3 < 67
    assert!(filename.len() < 255);
    
    filename
}