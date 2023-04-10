extern crate rex;

#[macro_use]
extern crate serde_derive;
extern crate serde_yaml;
extern crate bincode;

use std::convert::AsRef;
use std::path::Path;
use std::collections::BTreeMap;
use std::fs::File;
use std::io::BufReader;

use raqote::DrawTarget;
use rex::Renderer;

mod common;
use common::debug_render::{Equation, DebugRender};
use common::svg_diff;
use rex::cairo::CairoBackend;
use rex::font::FontContext;
use rex::font::backend::ttf_parser::TtfMathFont;
use rex::layout::{LayoutSettings, Style, Grid, Layout};
use rex::raqote::RaqoteBackend;

const LAYOUT_YAML: &str = "tests/data/layout.yaml";
const LAYOUT_HTML: &str = "tests/out/layout.html";
const LAYOUT_BINCODE: &str = "tests/data/layout.bincode";

#[derive(Debug, Serialize, Deserialize)]
struct Tests(BTreeMap<String, Vec<Category>>);

#[derive(Debug, Serialize, Deserialize)]
struct Category {
    #[serde(rename = "Description")]
    description: String,
    #[serde(rename = "Snippets")]
    snippets: Vec<String>,
}

fn collect_tests<P: AsRef<Path>>(path: P) -> Tests {
    let file = File::open(path.as_ref()).expect("failed to open test collection");
    let reader = BufReader::new(file);
    let tests: Tests = serde_yaml::from_reader(reader).expect("failed to parse test collection");

    tests
}

fn load_history<P: AsRef<Path>>(path: P) -> Vec<Equation> {
    let file = File::open(path.as_ref()).expect("failed to open test collection");
    let mut reader = BufReader::new(file);
    let tests: Vec<Equation> = bincode::deserialize_from(&mut reader)
        .expect("failed to load historical test results");

    tests
}

fn render_tests<'font, 'file>(ctx : &FontContext<'font, TtfMathFont<'file>>, tests: Tests) -> Vec<Equation> {
    let mut equations: Vec<Equation> = Vec::new();
    for (category, collection) in tests.0.iter() {
        for snippets in collection {
            for equation in &snippets.snippets {
                let equation = make_equation(category, &snippets.description, equation, ctx);
                equations.push(equation);
            }
        }
    }

    equations
}

fn make_equation(category: &str, description: &str, equation: &str, ctx: &FontContext<TtfMathFont>) -> Equation {
    const FONT_SIZE : f64 = 35.0;
    let description = format!("{}: {}", category, description);

    let parse_nodes = rex::parser::parse(equation).unwrap();
    let layout_settings = LayoutSettings::new(&ctx, FONT_SIZE, Style::Display);
    let mut grid = Grid::new();
    grid.insert(0, 0, rex::layout::engine::layout(&parse_nodes, layout_settings).unwrap().as_node());

    let mut layout = Layout::new();
    layout.add_node(grid.build());


    let renderer = Renderer::new();
    let (xmin, ymin, xmax, ymax) = renderer.size(&layout);
    let width  = xmax - xmin;
    let height = ymax - ymin;


    // debug rendering: gather drawing command issued
    let mut render = DebugRender::default();
    renderer.render(&layout, &mut render);

    // rendering to png
    let mut draw_target = DrawTarget::new(width.ceil() as i32, height.ceil() as i32);
    let mut raqote_backend = RaqoteBackend::new(&mut draw_target);
    renderer.render(&layout, &mut raqote_backend);

    // Unfortunate that we have to write to disk instead of outputting directly to stream
    // Raqote is limited in this way, cf [https://github.com/jrmuizel/raqote/pull/180]
    let path = std::env::temp_dir().join("out.png");

    // Some tests display empty stuff ; we don't want to panic then
    let img_render;
    if draw_target.write_png(&path).is_ok() {
        img_render = std::fs::read(&path).unwrap();
    }
    else {
        // Sometimes, a render is empty
        // Raqote throws an error but it is ok to ignore it
        // Problematically, we can't distinguish between an error safe to ignore (e.g. ZeroWidthError)
        // and one unsafe to ignore (e.g. IoError), b/c raqote doesn't give access to the underlying
        // png error type...
        img_render = include_bytes!("../resources/couldnt_render.png").to_vec();
    }

    // draw_target.write_png(&path).unwrap();




    Equation { 
        tex:         equation.to_owned(), 
        description, 
        width, height,
        render, 
        img_render, 
    }
}


fn equation_diffs(old: &[Equation], new: &[Equation]) -> Vec<(Equation, Equation)> {
    if old.len() != new.len() {
        panic!("Detected a change in the number of tests. Please be sure to run \
               `cargo test --test layout -- --ignored` to update the tests first.\n\
               Note: This should only be done before there are any changes which can alter \
               the result of a test.");
    }

    let mut diff: Vec<(Equation, Equation)> = Vec::new();
    for (left, right) in old.iter().zip(new.iter()) {
        if !left.same_as(right) {
            diff.push((left.clone(), right.clone()));
        }
    }

    diff
}

#[test]
fn layout() {
    let font_file : &[u8] = include_bytes!("../resources/XITS_Math.otf");
    let font = common::load_font(font_file);
    let font_context = FontContext::new(&font).unwrap();

    let tests = collect_tests(LAYOUT_YAML);
    let rendered = render_tests(&font_context, tests);
    let history = load_history(LAYOUT_BINCODE);
    let diff = equation_diffs(&history, &rendered);

    if diff.len() != 0 {
        let count = diff.len();
        svg_diff::write_diff(LAYOUT_HTML, &font_context, diff);
        panic!("Detected {} formula changes. \
                Please review the changes in `{}`",
               count,
               LAYOUT_HTML);
    }
}

#[test]
#[ignore]
fn save_layout() {
    use std::io::BufWriter;

    let font_file : &[u8] = include_bytes!("../resources/XITS_Math.otf");
    let font = common::load_font(font_file);
    let font_context = FontContext::new(&font).unwrap();

    // Load the tests in yaml, and render it to bincode
    let tests = collect_tests(LAYOUT_YAML);
    let rendered = render_tests(&font_context, tests);

    let out = File::create(LAYOUT_BINCODE).expect("failed to create bincode file for layout tests");
    let mut writer = BufWriter::new(out);
    bincode::serialize_into(&mut writer, &rendered)
        .expect("failed to serialize tex results to bincode");

}