//! Generate HTML report for regression tests

use base64::Engine;




use std::io::Write;
use std::path::Path;

use super::img_diff::diff_img;
use super::equation_sample::{Equation, EquationDiffs};

pub const HTML_REPORT_HEADER: &'static str =
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

pub const HTML_REPORT_FOOTER: &'static str = r"</body></html>";

fn write_equation_diff<W: Write>(f: &mut W, old: &Equation, new: &Equation) {
    write_equation_header(f, old);

    let engine = base64::engine::general_purpose::STANDARD_NO_PAD;

    let render_old = old.img_render_path.as_ref()
        .map(|path| std::fs::read(path).expect("Couldn't open file"));
    let render_new = new.img_render_path.as_ref()
        .map(|path| std::fs::read(path).expect("Couldn't open file"));




    let mut buffer_diff_img = Vec::new();
    // TODO this is first approximation ; we should hanle error cases here





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

    if let Some((old_img, new_img)) = Option::zip(render_old, render_new) {
        diff_img(&old_img, &new_img, &mut buffer_diff_img);
        writeln!(
            f,
            r#"
            <table class="diff-array">
            <thead><tr><td>Diff</td></tr></thead>
            <tbody><tr><td><img src="data:image/png;base64,{}"></td></tr></tbody>
            </table>
            "#,
            engine.encode(&buffer_diff_img),
        ).unwrap();
    }
    else {
        writeln!(
            f,
            "<div><strong>One of the image was an empty render.</strong></div>"
        ).unwrap();
    }

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



pub fn write_diff<P: AsRef<Path>>(path: P, diff: EquationDiffs) {
    use std::fs::File;
    use std::io::BufWriter;

    let out = File::create(path.as_ref()).expect("failed to create html file for SVG diff");
    let mut writer = BufWriter::new(out);

    writer.write(HTML_REPORT_HEADER.as_bytes()).unwrap();
    for new_eq in diff.new_eqs {
        write_equation(&mut writer, new_eq);
    }
    for (before, after) in diff.diffs {
        write_equation_diff(&mut writer, before, after);
    }
    writer.write(HTML_REPORT_FOOTER.as_bytes()).unwrap();
}
