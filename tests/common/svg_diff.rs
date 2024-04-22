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
    .content {
        display: none;
    }
    </style>
</head>
<body>"##;

const END: &'static str = r##"
<script>
    var coll = document.getElementsByClassName("collapsible");
    var i;

    for (i = 0; i < coll.length; i++) {
      coll[i].addEventListener("click", function() {
        this.classList.toggle("active");
        var content = this.nextElementSibling;
        if (content.style.display === "block") {
          content.style.display = "none";
        } else {
          content.style.display = "block";
        }
      });
    } 
</script>
</body>
</html>"##;

fn write_equation_diff<W: Write>(f: &mut W, old: &Equation, new: &Equation) {
    write_equation_header(f, old);

    let engine = base64::engine::general_purpose::STANDARD_NO_PAD;

    let render_old = old.render.as_ref().ok()
        .and_then(|render| render.img_render_path.as_ref())
        .map(|path| std::fs::read(path).expect("Couldn't open file"));
    let render_new = new.render.as_ref().ok()
        .and_then(|render| render.img_render_path.as_ref())
        .map(|path| std::fs::read(path).expect("Couldn't open file"));





    // TODO this is first approximation ; we should hanle error cases here



    let diff_raw = diff_debug(&old.render, &new.render);


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
        "#,
        engine.encode(render_old.as_ref().map(Vec::as_slice).unwrap_or_else(|| include_bytes!("../../resources/couldnt_render.png"))),
        engine.encode(render_new.as_ref().map(Vec::as_slice).unwrap_or_else(|| include_bytes!("../../resources/couldnt_render.png"))),
    ).unwrap();

    if let Err(e) = &new.render {
        writeln!(f, r#"<p>New failed to render, returned error: <pre>{}</pre></p>"#, e).unwrap();
    }
    if let Err(e) = &old.render {
        writeln!(f, r#"<p>Old failed to render, returned error: <pre>{}</pre></p>"#, e).unwrap();
    }
    

    if let Some((old_img, new_img)) = Option::zip(render_old, render_new) {
        writeln!(
            f,
            r#"
            <table class="diff-array">
            <thead><tr><td>Diff</td></tr></thead>
            <tbody><tr><td><img src="data:image/png;base64,{}"></td></tr></tbody>
            </table>
            "#,
            engine.encode(&diff_img(&old_img, &new_img)),
        ).unwrap();
    }
    else {
        writeln!(
            f,
            "<div><strong>One of the image was an empty render.</strong></div>"
        ).unwrap();
    }

    writeln!(f, r#"
        <button type="button" class="collapsible">See raw diff between renders</button>
        <pre class="content">{}</pre>
    "#,
        diff_raw
    ).unwrap();

}

fn diff_debug<D : std::fmt::Debug>(x : &D, y : &D) -> String {
    let debug_render_x = format!("{:#?}", x);
    let debug_render_y = format!("{:#?}", y);
    let text_diff = similar::TextDiff::from_lines(
        &debug_render_x,
        &debug_render_y,
    );

    let mut to_return = String::new();

    for change in text_diff.iter_all_changes() {
        let sign = match change.tag() {
            similar::ChangeTag::Delete => "-",
            similar::ChangeTag::Insert => "+",
            similar::ChangeTag::Equal  => "=",
        };
        to_return.push_str(
            &format!("{}{}", sign, change)
        );
    }
    to_return
}

fn write_equation<W: Write>(f: &mut W, eq: &Equation,) {
    write_equation_header(f, eq);

    let render : Vec<u8>;
    if let Some(path) = eq.render.as_ref().ok().and_then(|render| render.img_render_path.as_ref()) {
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
