use std::io::Cursor;

use image::{ImageOutputFormat, Rgba};



pub fn diff_img(bytes_before_img : &[u8], bytes_after_img : &[u8], buffer_diff_img : &mut Vec<u8>) -> bool {
    let before_img = image::io::Reader::new(Cursor::new(bytes_before_img))
        .with_guessed_format().unwrap()
        .decode().unwrap()
    ;
    let before_img = before_img.to_rgba8();
    let after_img = image::io::Reader::new(Cursor::new(bytes_after_img))
        .with_guessed_format().unwrap()
        .decode().unwrap()
    ;
    let after_img = after_img.as_rgba8().unwrap();

    let (before_width, before_height) = before_img.dimensions();
    let (after_width,  after_height)  = after_img.dimensions();

    let diff_width  = std::cmp::max(before_width, after_width); 
    let diff_height = std::cmp::max(before_height, after_height); 

    let mut diff_img = image::RgbaImage::new(diff_width, diff_height);
    let mut has_diff = false;
    for (x, y, pixel) in diff_img.enumerate_pixels_mut() {
        let before_pixel = before_img.get_pixel_checked(x, y);
        let after_pixel  = after_img.get_pixel_checked(x, y);
        const INSERTION_COLOR   : Rgba<u8> = Rgba([0x00, 0xff, 0x00, 0xff,]);
        const DELETION_COLOR    : Rgba<u8> = Rgba([0xff, 0x00, 0x00, 0xff,]);
        const BACKGROUND_COLOR  : Rgba<u8> = Rgba([0x00, 0x00, 0x00, 0x00,]);

        if before_pixel == after_pixel {
            if let Some(pixel_to_place) = after_pixel {
                *pixel = *pixel_to_place;
            }
            else {
                *pixel = Rgba([0xff, 0xff, 0xff, 0xff,]);
            }
        }
        else if let Some((before_pixel, after_pixel)) = Option::zip(before_pixel, after_pixel) {
            // if both are transparent, no matter the underlying color, then treat them as the same
            if before_pixel.0[3] == 0 && after_pixel.0[3] == 0 {
                *pixel = *before_pixel;
            }
            else {
                has_diff = true;
                fn to_value(pixel : &Rgba<u8>) -> u32 {
                    // no matter what, a non-transparent pixel should have a higher value than a transparent one
                    (u32::from(pixel.0[0]) + u32::from(pixel.0[1]) +
                    u32::from(pixel.0[2]) + 1) * u32::from(pixel.0[3])
                }
                // if there's less color after than before, color in red (it's a "deletion")
                if to_value(after_pixel) < to_value(before_pixel) {
                    *pixel = DELETION_COLOR;
                }
                // otherwise in green, it's an insertion
                else {
                    *pixel = INSERTION_COLOR;
                }
            }
        }
        else if let Some(before_pixel) = before_pixel {
            has_diff = true;
            if before_pixel.0[3] == 0x00 {
                *pixel = BACKGROUND_COLOR
            }
            else {
                *pixel = DELETION_COLOR
            }
        }
        else if let Some(after_pixel) = after_pixel {
            has_diff = true;
            if after_pixel.0[3] == 0x00 {
                *pixel = BACKGROUND_COLOR
            }
            else {
                *pixel = INSERTION_COLOR
            }
        }
    } 

    buffer_diff_img.clear();
    let mut to_return = Cursor::new(buffer_diff_img);
    diff_img.write_to(&mut to_return, ImageOutputFormat::Png).unwrap();

    has_diff
}
