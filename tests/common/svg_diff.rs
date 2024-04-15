use base64::Engine;
use image::{Rgba, ImageOutputFormat};

use super::debug_render::{Equation, EquationDiffs};

use std::io::{Write, Cursor};
use std::path::Path;

const HEADER: &'static str =
r##"<!DOCTYPE html>
<html>
<head>
    <meta charset="UTF-8">
    <title>ReX layout tests</title>
    <link rel="stylesheet" href="https://cdnjs.cloudflare.com/ajax/libs/prism/1.6.0/themes/prism-okaidia.min.css"/>
    <link rel="shortcut icon" href="rex.ico"/>
    <script src="https://cdnjs.cloudflare.com/ajax/libs/prism/1.6.0/prism.min.js"></script>
    <script src="https://cdnjs.cloudflare.com/ajax/libs/prism/1.6.0/components/prism-latex.min.js"></script>
    <style>
    .diff-array {
        margin: auto;
    }
    .diff-array thead {
        font-weight: bold;
    }
    .diff-array td {
        text-align: center;
        padding: 5px;
    }
    .diff-array img {
        border: solid 1pt black;
    }
    </style>
</head>
<body>"##;

const END: &'static str = r"</body></html>";

fn write_equation_diff<W: Write>(f: &mut W, old: &Equation, new: &Equation) {
    write_equation_header(f, old);


    // TODO this is first approximation ; we should hanle error cases here
    let render_old = std::fs::read(old.img_render_path.as_ref().unwrap().as_path()).unwrap();
    let render_new = std::fs::read(new.img_render_path.as_ref().unwrap().as_path()).unwrap();


    let engine = base64::engine::general_purpose::STANDARD_NO_PAD;

    writeln!(
        f,
        r#"
        <table class="diff-array">
        <thead><tr>
        <td>Old</td>
        <td>New</td>
        </tr></thead>
        <tbody>
        <tr>
        <td><img src="data:image/png;base64,{}"></td>
        <td><img src="data:image/png;base64,{}"></td>
        </tr>
        </tbody>
        </table>
        <table class="diff-array">
        <thead><tr><td>Diff</td></tr></thead>
        <tbody><tr><td><img src="data:image/png;base64,{}"></td></tr></tbody>
        </table>
        "#,
        engine.encode(&render_old),
        engine.encode(&render_new),
        engine.encode(&diff_img(&render_old, &render_new)),
    ).unwrap();

}

fn write_equation<W: Write>(f: &mut W, eq: &Equation,) {
    write_equation_header(f, eq);

    let render : Vec<u8>;
    if let Some(path) = eq.img_render_path.as_ref() {
        eprintln!("{}", path.as_os_str().to_str().unwrap());
        render = std::fs::read(path).unwrap();
    }
    else {
        render = include_bytes!("../../resources/couldnt_render.png").to_vec();
    }

    let engine = base64::engine::general_purpose::STANDARD_NO_PAD;

    writeln!(
        f, 
        r#"
        <table class="diff-array">
        <thead><tr>
        <td>New test</td>
        </tr></thead>
        <tbody>
        <tr>
        <td><img src="data:image/png;base64,{}"></td>
        </tr>
        </tbody>
        </table>
        "#,
        engine.encode(render),
    ).unwrap();

}



fn write_equation_header<W : Write>(f: &mut W, equation: &Equation) {
    writeln!(f, "<h2>{}</h2>", equation.description).unwrap();
    writeln!(f,
             r#"<pre><code class="language-latex">{}</code></pre>"#,
             equation.tex)
            .unwrap();
}


fn diff_img(bytes_before_img : &[u8], bytes_after_img : &[u8]) -> Vec<u8> {
    let before_img = image::io::Reader::new(Cursor::new(bytes_before_img))
        .with_guessed_format().unwrap()
        .decode().unwrap()
    ;
    let before_img = before_img.as_rgba8().unwrap();
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
            fn to_value(pixel : &Rgba<u8>) -> u32 {
                u32::from(pixel.0[0]) + u32::from(pixel.0[1]) +
                u32::from(pixel.0[2]) + u32::from(pixel.0[3])
            }
            // if there's more color after than before, color in green (it's an "insertion")
            if to_value(after_pixel) < to_value(before_pixel) {
                *pixel = DELETION_COLOR;
            }
            // otherwise in green
            else {
                *pixel = INSERTION_COLOR;
            }
        }
        else if let Some(before_pixel) = before_pixel {
            if before_pixel.0[3] == 0x00 {
                *pixel = BACKGROUND_COLOR
            }
            else {
                *pixel = DELETION_COLOR
            }
        }
        else if let Some(after_pixel) = after_pixel {
            if after_pixel.0[3] == 0x00 {
                *pixel = BACKGROUND_COLOR
            }
            else {
                *pixel = INSERTION_COLOR
            }
        }
    } 

    let mut to_return = Cursor::new(Vec::new());
    diff_img.write_to(&mut to_return, ImageOutputFormat::Png).unwrap();
    
    to_return.into_inner()
}

pub fn write_diff<P: AsRef<Path>>(path: P, diff: EquationDiffs) {
    use std::fs::File;
    use std::io::BufWriter;

    let out = File::create(path.as_ref()).expect("failed to create html file for SVG diff");
    let mut writer = BufWriter::new(out);

    writer.write(HEADER.as_bytes()).unwrap();
    for new_eq in diff.new_eqs {
        write_equation(&mut writer, new_eq);
    }
    for (before, after) in diff.diffs {
        write_equation_diff(&mut writer, before, after);
    }
    writer.write(END.as_bytes()).unwrap();
}
