//! Regression tests
extern crate rex;

#[macro_use]
extern crate serde_derive;
extern crate serde_yaml;

use std::convert::AsRef;
use std::path::Path;
use std::collections::BTreeMap;
use std::fs::File;
use std::io::BufReader;


use raqote::{DrawTarget, Transform};
use rex::Renderer;

mod common;
use common::debug_render::DebugRender;
use common::equation_sample::{Equation, EquationDiffs, EquationRender};
use common::utils::equation_to_filepath;
use rex::font::FontContext;
use rex::font::backend::ttf_parser::TtfMathFont;
use rex::layout::{LayoutSettings, Style};
use rex::raqote::RaqoteBackend;

const REGRESSION_RENDER_YAML: &str = "tests/data/regression_render.yaml";
const REGRESSION_RENDER_OUT_HTML: &str = "tests/out/regression_render.html";

const HISTORY_META_FILE: &str = "tests/data/history_regression_render.yaml";
const HISTORY_IMG_DIR:   &str = "tests/data/imgs/";

#[derive(Debug, Serialize, Deserialize)]
struct Tests(BTreeMap<String, Vec<Category>>);

#[derive(Debug, Serialize, Deserialize)]
struct Category {
    #[serde(rename = "Description")]
    description: String,
    #[serde(rename = "Snippets")]
    snippets: Vec<String>,
}

type TestResults = BTreeMap<String, Equation>;

fn collect_tests<P: AsRef<Path>>(path: P) -> Tests {
    let file = File::open(path.as_ref()).expect("failed to open test collection");
    let reader = BufReader::new(file);
    let tests: Tests = serde_yaml::from_reader(reader).expect("failed to parse test collection");

    tests
}

fn load_history<P: AsRef<Path>>(path: P) -> TestResults {
    let file = File::open(path.as_ref()).expect("failed to open test collection");
    let mut reader = BufReader::new(file);
    let tests: TestResults = serde_yaml::from_reader(&mut reader)
        .expect("history file not understood - did you change the structure used for tests?");

    tests
}


fn render_tests<'font, 'file>(ctx : &FontContext<'font, TtfMathFont<'file>>, tests: Tests, img_dir : &Path) -> TestResults {

    let mut equations: TestResults = BTreeMap::new();
    for (category, collection) in tests.0.iter() {
        for snippets in collection {
            for equation in &snippets.snippets {
                let key = format!("{} - {}", &snippets.description, equation);
                let file_name = equation_to_filepath(equation, &snippets.description);
                let img_path = img_dir.join(&file_name);
                let equation = make_equation(
                    category, 
                    &snippets.description, 
                    equation, 
                    ctx,
                    &img_path,
                );
                equations.insert(key, equation);
            }
        }
    }

    equations
}



fn make_equation(
    category: &str, 
    description: &str, 
    equation: &str, 
    ctx: &FontContext<TtfMathFont>,
    img_render_path : &Path,
) -> Equation {
    let description = format!("{}: {}", category, description);

    let render = render_equation(equation, ctx, img_render_path);

    Equation { 
        tex:         equation.to_owned(), 
        description,
        render, 
    }
}

fn render_equation(equation: &str, ctx: &FontContext<'_, TtfMathFont<'_>>, img_render_path: &Path) -> Result<EquationRender, String> {
    const FONT_SIZE : f64 = 16.0;
    // let description = format!("{}: {}", category, description);

    let parse_nodes = rex::parser::parse(equation).map_err(|e| e.to_string())?;
    let layout_settings = LayoutSettings::new(&ctx).font_size(FONT_SIZE).layout_style(Style::Display);

    let layout = rex::layout::engine::layout(&parse_nodes, layout_settings).unwrap();


    let renderer = Renderer::new();
    let dims = layout.size();
    let width  = dims.width;
    let height = dims.height - dims.depth;


    // debug rendering: gather drawing command issued
    let mut debug_render = DebugRender::default();
    renderer.render(&layout, &mut debug_render);

    // rendering to png
    const SCALE: f32 = 2.5;
    let mut draw_target = DrawTarget::new((width.ceil() * f64::from(SCALE)) as i32, (height.ceil() * f64::from(SCALE)) as i32);
    draw_target.set_transform(&Transform::translation(0., dims.height as f32).then_scale(SCALE, SCALE));
    let mut raqote_backend = RaqoteBackend::new(&mut draw_target);
    renderer.render(&layout, &mut raqote_backend);


    let final_image_path_buffer;
    // Some tests display empty stuff ; we don't want to panic then
    match draw_target.write_png(&img_render_path) {
        Ok(_) => {
            final_image_path_buffer = Some(img_render_path.to_owned());
        },
        Err(e) => {
            // Sometimes, a render is empty
            // Raqote throws an error but it is ok to ignore empty renders
            // Problematically, we can't distinguish between an error safe to ignore (e.g. ZeroWidthError)
            // and one unsafe to ignore (e.g. IoError), b/c raqote doesn't give access to the underlying
            // png error type...
            eprintln!("Error writing png: {}", e);
            eprintln!("png name: {}", img_render_path.to_str().unwrap());
            final_image_path_buffer = None;
        }
    }

    Ok(EquationRender {
        width,
        height,
        render: debug_render,
        img_render_path: final_image_path_buffer,
    })
}


macro_rules! function_name {
    ($func:expr) => {{
        // Force the function to be used so the compiler checks if it exists
        let _ = $func as fn();
        stringify!($func)
    }};
}


fn equation_diffs<'a>(old: &'a TestResults, new: &'a TestResults) -> EquationDiffs<'a> {
    if old.len() != new.len() {
        eprintln!("Detected a change in the number of tests. Please be sure to run \
               `cargo test --test {} -- --ignored` to update the test history.", function_name!(save_renders_to_history));
    }

    let mut diffs: Vec<(&'a Equation, &'a Equation)> = Vec::new();
    let mut new_eqs = Vec::new();

    // Only looking at tests in the intersection of both
    for (key_new, equation_new) in new.iter() {
        if let Some(equation_old) = old.get(key_new) {
            if !equation_old.same_render_as(equation_new) {
                diffs.push((equation_old, equation_new))
            }
        }
        else {
            new_eqs.push(equation_new)
        }
    }

    EquationDiffs {
        diffs,
        new_eqs,
    }
}

#[test]
fn render_regression() {
    let font_file : &[u8] = include_bytes!("../resources/XITS_Math.otf");
    let font = common::utils::load_font(font_file);
    let font_context = FontContext::new(&font);

    let img_dir = std::env::temp_dir();
    let tests = collect_tests(REGRESSION_RENDER_YAML);
    let rendered = render_tests(&font_context, tests, img_dir.as_path());
    let history = load_history(HISTORY_META_FILE);
    let diff = equation_diffs(&history, &rendered);

    if !diff.no_diff() {
        let diff_count = diff.diffs.len();
        let new_count  = diff.new_eqs.len();
        common::report::write_diff(REGRESSION_RENDER_OUT_HTML, diff);
        panic!("Detected {} formula changes and {} new formulas. \
                Please review the changes in `{}`",
               diff_count,
               new_count,
               REGRESSION_RENDER_OUT_HTML);
    }
}

#[test]
#[ignore]
fn save_renders_to_history() {
    use std::io::BufWriter;

    let font_file : &[u8] = include_bytes!("../resources/XITS_Math.otf");
    let font = common::utils::load_font(font_file);
    let font_context = FontContext::new(&font);

    // Remove PNG images from HISTORY_IMG_DIR
    let img_dir = Path::new(HISTORY_IMG_DIR);
    for entry in std::fs::read_dir(img_dir).expect("failed to read image directory") {
        if let Ok(entry) = entry {
            let file_path = entry.path();
            if let Some(extension) = file_path.extension() {
                if extension == "png" {
                    std::fs::remove_file(file_path).expect("failed to remove PNG image");
                }
            }
        }
    }

    // Load the tests in yaml, and render it to bincode
    let tests = collect_tests(REGRESSION_RENDER_YAML);
    let rendered = render_tests(&font_context, tests, img_dir);

    let out = File::create(HISTORY_META_FILE).expect("failed to create bincode file for layout tests");
    let writer = BufWriter::new(out);
    serde_yaml::to_writer(writer, &rendered)
        .expect("failed to serialize tex results to bincode");
}
