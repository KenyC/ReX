use std::{io::Write, path::PathBuf, process::Command};

use base64::Engine;
use raqote::{DrawTarget, Transform};
use rex::{font::{backend::ttf_parser::TtfMathFont, FontContext}, layout::{engine::layout, LayoutSettings}, raqote::RaqoteBackend, Renderer};

mod common;
use common::report::HTML_REPORT_FOOTER;

use crate::common::{img_diff::diff_img, report::HTML_REPORT_HEADER};


const SAMPLES_PATH   : &str = "tests/data/tex_comparison_samples.json";
const OUT_PATH       : &str = "tests/out/tex_comparison_report.html";
const TEX_RENDER_DIR : &str = "tests/out/text-renders";
// Must be one of document class accepted font sizes, i.e.
// 10, 11 or 12 pt.em-1
const FONT_SIZE      : f64  = 10.0;
// We still want to see details so we scale the output
const SCALE_RENDER : f32 = 3.;

const MATH_FONT : &[u8] = include_bytes!("../resources/XITS_Math.otf");


// fn main() {
//     let font = ttf_parser::Face::parse(MATH_FONT, 0).unwrap();
//     let math_font = TtfMathFont::new(font).unwrap();
//     let font_context = FontContext::new(&math_font);

//     let content = compile_rex_sample("1+1=2", &font_context).unwrap();

//     std::fs::write("/tmp/ex.png", &content).unwrap();
// }

#[test]
#[ignore]
fn tex_comparison() {
   let samples_file = std::fs::File::open(SAMPLES_PATH).expect("Couldn't open samples file");
   let samples : Vec<String> = serde_json::from_reader(samples_file).expect("Couldn't parse samples file");

   let mut out_file = std::fs::File::create(OUT_PATH).expect("Couldn't create out path");
   let base64_engine = base64::engine::general_purpose::STANDARD_NO_PAD;


   let font = ttf_parser::Face::parse(MATH_FONT, 0).unwrap();
   let math_font = TtfMathFont::new(font).unwrap();
   let font_context = FontContext::new(&math_font);

   let mut buffer_diff_img = Vec::new();
   let mut no_diff_rex_and_tex = true;

   out_file.write(HTML_REPORT_HEADER.as_bytes()).expect("Couldn't write to file");
   for sample in samples {
        let tex_sample_path = sample_to_filepath(&sample);
        let tex_render :Result<Vec<u8>, TexCompilationError> = 
            std::fs::read(&tex_sample_path)
            .or_else(|_| {
                let tex_sample = compile_tex_sample(&sample)?;
                std::fs::write(&tex_sample_path, &tex_sample).unwrap_or(());
                Ok(tex_sample)
            })
        ;
        let rex_render = compile_rex_sample(&sample, &font_context);


        // Write TeX render
        out_file.write(
            "<table class=\"diff-array\"><thead><tr><td>TeX</td><td>ReX</td><td>Diff</td></tr></thead><tbody><tr>".as_bytes()
        ).expect("Couldn't write to file");

        out_file.write("<td>".as_bytes()).expect("Couldn't write to file");
        match &tex_render {
            Ok(render)  => write!(out_file, r#"<img src="data:image/png;base64,{}">"#, base64_engine.encode(&render)),
            Err(error)  => write!(out_file, "LaTeX render error: {:?}", error),
        }.expect("Couldn't write to file");



        // Write ReX render
        out_file.write("</td><td>".as_bytes()).expect("Couldn't write to file");
        match &rex_render {
            Ok(render)  => 
                write!(out_file, r#"<img src="data:image/png;base64,{}">"#, base64_engine.encode(&render)),
            Err(error)      => 
                write!(out_file, "ReX render error: {:?}", error),

        }.expect("Couldn't write to file");
        out_file.write("</td><td>".as_bytes()).expect("Couldn't write to file");

        if tex_render.is_ok() != rex_render.is_ok() {
            no_diff_rex_and_tex = false;
        }

        // Write diff
        match Option::zip(tex_render.ok(), rex_render.ok()) {
            Some((tex_render, rex_render)) => {
                let diff_img = diff_img(&tex_render, &rex_render, &mut buffer_diff_img);
                no_diff_rex_and_tex = no_diff_rex_and_tex && !diff_img;
                write!(out_file, r#"<img src="data:image/png;base64,{}">"#, base64_engine.encode(&buffer_diff_img))  
            },
            None => 
                out_file.write("Problem with one of the renders".as_bytes()).map(|_| ())
        }.expect("Couldn't write to file");
        out_file.write("</td>".as_bytes()).expect("Couldn't write to file");


        out_file.write("</tr> </tbody> </table>".as_bytes()).expect("Couldn't write to file");
   }
   out_file.write(HTML_REPORT_FOOTER.as_bytes()).expect("Couldn't write to file");

   eprintln!("Comparison file written to {}", OUT_PATH);
   assert!(no_diff_rex_and_tex);
}

fn compile_rex_sample<'a>(sample: &str, font_context : & FontContext<'a, TtfMathFont>) -> Result<Vec<u8>, rex::error::Error> {
    // parse
    let nodes = rex::parser::parse(sample)?;

    let layout_settings = LayoutSettings::new(font_context)
        .font_size(FONT_SIZE)
        .layout_style(rex::layout::Style::Display)
    ;
    let layout = layout(&nodes, layout_settings)?;


    let renderer = Renderer::new();
    let dims = layout.size();
    let width  = dims.width;
    let height = dims.height - dims.depth;


    // rendering to png
    let mut draw_target = DrawTarget::new((width.ceil() * f64::from(SCALE_RENDER)) as i32, (height.ceil() * f64::from(SCALE_RENDER)) as i32);
    draw_target.set_transform(&Transform::translation(0., dims.height as f32).then_scale(SCALE_RENDER, SCALE_RENDER));
    let mut raqote_backend = RaqoteBackend::new(&mut draw_target);
    renderer.render(&layout, &mut raqote_backend);


    let png_target = std::env::temp_dir().join("rex_png_render.png");
    draw_target.write_png(&png_target).unwrap();

    Ok(std::fs::read(&png_target).unwrap())
}

#[derive(Debug)]
enum TexCompilationError {
    IoError(std::io::Error),
    CompilationFailed(String),
    ConversionToPngFailed(String),
    CroppingFailed(String),
}



fn compile_tex_sample(sample: &str) -> Result<Vec<u8>, TexCompilationError> {
    let temp_dir_path : PathBuf = create_temp_dir().map_err(TexCompilationError::IoError)?;

    std::fs::write(temp_dir_path.join("mathfont.otf"), MATH_FONT).map_err(TexCompilationError::IoError)?;

    let tex_file = format!(r#"
    \documentclass[{}pt]{{article}}
    \usepackage{{unicode-math}} % to use unicode in the formulas -- to improve readability of sources
    \usepackage{{xcolor}}
    \setmathfont{{mathfont.otf}} % it is important to have this line after the amsmath, mathtools and other maths
    \pagestyle{{empty}}
    \begin{{document}}
    $${}$$
    \end{{document}}
    "#, 
        FONT_SIZE.round() as i64,
        sample
    );

    let tex_file_path = temp_dir_path.join("main.tex");
    std::fs::write(&tex_file_path, tex_file).map_err(TexCompilationError::IoError)?;


    let mut tex_process = Command::new("xelatex");
    tex_process.current_dir(&temp_dir_path);
    tex_process.arg(tex_file_path.as_os_str());
    tex_process.arg("-interaction=nonstopmode");

    let output = tex_process.output().map_err(TexCompilationError::IoError)?;
    if !output.status.success(){
        return Err(TexCompilationError::CompilationFailed(String::from_utf8(output.stdout).unwrap()));
    } 

    let pdf_path = tex_file_path.with_extension("pdf");
    let unknown_format_png_path = pdf_path.with_extension("png");
    let cropped_pdf_path = pdf_path.with_file_name("crop.pdf");

    let mut crop_process = Command::new("pdfcrop");
    crop_process.arg(pdf_path.as_os_str());
    crop_process.arg(&cropped_pdf_path);


    let output = crop_process.output().map_err(TexCompilationError::IoError)?;
    if !output.status.success(){
        return Err(TexCompilationError::CroppingFailed(String::from_utf8(output.stderr).unwrap()));
    } 

    let mut convert_process = Command::new("convert");
    convert_process.arg("-density");
    convert_process.arg(format!("{}", (SCALE_RENDER * 96.).round() as i64));
    convert_process.arg("-background");
    convert_process.arg("white");
    convert_process.arg(cropped_pdf_path.as_os_str());
    convert_process.arg(unknown_format_png_path.as_os_str());


    let output = convert_process.output().map_err(TexCompilationError::IoError)?;
    if !output.status.success(){
        return Err(TexCompilationError::ConversionToPngFailed(String::from_utf8(output.stderr).unwrap()));
    } 


    std::fs::read(unknown_format_png_path).map_err(TexCompilationError::IoError)
}

fn create_temp_dir() -> Result<PathBuf, std::io::Error> {
    let path = std::env::temp_dir().join("rex-latex-comp");
    std::fs::DirBuilder::new().recursive(true).create(&path)?;
    Ok(path)
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


pub fn sample_to_filepath(equation: &str) -> PathBuf {
    let bytes = equation.as_bytes().to_vec();

    let filename = format!(
        "{}-{}.png",
        &small_ascii_repr(equation),
        base64::engine::general_purpose::URL_SAFE_NO_PAD.encode(simple_hash(&bytes)),
    );

    PathBuf::from(TEX_RENDER_DIR).join(filename)
}