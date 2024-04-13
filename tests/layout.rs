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

use raqote::{DrawTarget, Transform};
use rex::Renderer;

mod common;
use common::debug_render::{Equation, DebugRender, EquationDiffs};
use common::svg_diff;
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

fn load_history<P: AsRef<Path>>(path: P) -> BTreeMap<String, Equation> {
    let file = File::open(path.as_ref()).expect("failed to open test collection");
    let mut reader = BufReader::new(file);
    let tests: BTreeMap<String, Equation> = bincode::deserialize_from(&mut reader)
        .expect("history file not understood - did you change the structure used for tests?");

    tests
}

fn render_tests<'font, 'file>(ctx : &FontContext<'font, TtfMathFont<'file>>, tests: Tests) -> BTreeMap<String, Equation> {
    let mut equations: BTreeMap<String, Equation> = BTreeMap::new();
    for (category, collection) in tests.0.iter() {
        for snippets in collection {
            for equation in &snippets.snippets {
                let equation = make_equation(category, &snippets.description, equation, ctx);
                let key = format!("{} - {}", equation.description, equation.tex);
                equations.insert(key, equation);
            }
        }
    }

    equations
}

fn make_equation(category: &str, description: &str, equation: &str, ctx: &FontContext<TtfMathFont>) -> Equation {
    const FONT_SIZE : f64 = 16.0;
    let description = format!("{}: {}", category, description);

    let parse_nodes = rex::parser::parse(equation).expect(&format!("Error with {}", equation));
    let layout_settings = LayoutSettings::new(&ctx, FONT_SIZE, Style::Display);
    let mut grid = Grid::new();
    grid.insert(0, 0, rex::layout::engine::layout(&parse_nodes, layout_settings).unwrap().as_node());

    let mut layout = Layout::new();
    layout.add_node(grid.build());


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
    draw_target.set_transform(&Transform::scale(SCALE, SCALE));
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
        // Raqote throws an error but it is ok to ignore empty renders
        // Problematically, we can't distinguish between an error safe to ignore (e.g. ZeroWidthError)
        // and one unsafe to ignore (e.g. IoError), b/c raqote doesn't give access to the underlying
        // png error type...
        img_render = include_bytes!("../resources/couldnt_render.png").to_vec();
    }


    Equation { 
        tex:         equation.to_owned(), 
        description, 
        width, height,
        render: debug_render, 
        img_render, 
    }
}



fn equation_diffs<'a>(old: &'a BTreeMap<String, Equation>, new: &'a BTreeMap<String, Equation>) -> EquationDiffs<'a> {
    if old.len() != new.len() {
        eprintln!("Detected a change in the number of tests. Please be sure to run \
               `cargo test --test layout -- --ignored` to update the test history.");
    }

    let mut diffs: Vec<(&'a Equation, &'a Equation)> = Vec::new();
    let mut new_eqs = Vec::new();

    // Only looking at tests in the intersection of both
    for (key_new, equation_new) in new.iter() {
        if let Some(equation_old) = old.get(key_new) {
            if !equation_old.same_as(equation_new) {
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
fn layout() {
    let font_file : &[u8] = include_bytes!("../resources/XITS_Math.otf");
    let font = common::load_font(font_file);
    let font_context = FontContext::new(&font).unwrap();

    let tests = collect_tests(LAYOUT_YAML);
    let rendered = render_tests(&font_context, tests);
    let history = load_history(LAYOUT_BINCODE);
    let diff = equation_diffs(&history, &rendered);

    if !diff.no_diff() {
        let diff_count = diff.diffs.len();
        let new_count  = diff.new_eqs.len();
        svg_diff::write_diff(LAYOUT_HTML, diff);
        panic!("Detected {} formula changes and {} new formulas. \
                Please review the changes in `{}`",
               diff_count,
               new_count,
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