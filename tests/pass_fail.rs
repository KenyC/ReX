extern crate rex;
#[macro_use]
extern crate serde_derive;
extern crate serde_yaml;

use std::fs::File;
use std::io::BufReader;

use rex::Renderer;
use rex::cairo::CairoBackend;
use rex::error::Error;
use rex::font::FontContext;
use rex::font::backend::ttf_parser::TtfMathFont;
use rex::layout::{LayoutSettings, Style, Grid};
use rex::parser::parse;
//use std::io::Sink;

const FONT_FILE_PATH : &str = "resources/Garamond_Math.otf";


#[derive(Debug, Serialize, Deserialize)]
struct Tests {
    #[serde(rename="Pass")]
    pass: Vec<String>,
    #[serde(rename="Fail")]
    fail: Vec<String>,
}

#[test]
fn pass_fail() {
    let file = File::open("tests/data/passfail.yaml").expect("failed to open passfail yaml");
    let reader = BufReader::new(file);
    let tests: Tests = serde_yaml::from_reader(reader).expect("failed to parse passfail.yaml");
    let mut fail = 0;

    let font_file = std::fs::read(FONT_FILE_PATH).unwrap();
    let font = load_font(&font_file);
    let ctx = FontContext::new(&font).unwrap();
    


    for test in tests.pass {
        match render(&ctx, &test) {
            Ok(_) => continue,
            Err(err) => {
                println!("Tex: {}", test);
                println!("Should have passed, failed with: {:?}", err);
                fail += 1;
            }
        }
    }

    // TODO: We need to stop panicking and pass the errors properly.
    // TODO: We need to use io::Write instead of fmt::Write for the
    //       rendering traits.
    //
    // for test in tests.fail {
    //     match SvgSink::new(&settings).render(&test) {
    //         Err(_) => continue,
    //         Ok(_) => {
    //             println!("Tex: {}", test);
    //             println!("Should have failed");
    //             fail += 1;
    //         }
    //     }
    // }

    if fail > 0 {
        panic!("{} Pass/Fail tests failed.", fail);
    }
}

fn render<'a, 'f, 'b>(ctx : &FontContext<'f, TtfMathFont<'a>>, string : &'b str) -> Result<String, Error> {
    // parsing
    let parse_nodes = parse(string)?;

    // laying out
    let layout_settings = LayoutSettings::new(&ctx, 10.0, Style::Display);
    let node = rex::layout::engine::layout(&parse_nodes, layout_settings).map(|l| l.as_node())?;
    let mut grid = Grid::new();
    grid.insert(0, 0, node);
    let mut layout = rex::layout::Layout::new();
    layout.add_node(grid.build());


    // Sizing
    let renderer = Renderer::new();
    let formula_bbox = layout.size();


    // Rendering
    let mut svg_bytes : Vec<u8> = Vec::new();
    let svg_surface = unsafe { cairo::SvgSurface::for_raw_stream(
        formula_bbox.width, 
        formula_bbox.height - formula_bbox.depth, 
        &mut svg_bytes,
    )}.unwrap();
    let context = cairo::Context::new(&svg_surface).unwrap();
    let mut backend = CairoBackend::new(context);
    renderer.render(&layout, &mut backend);
    svg_surface.finish_output_stream().unwrap();

    Ok(String::from_utf8(svg_bytes).unwrap())
}


fn load_font<'a>(file : &'a [u8]) -> rex::font::backend::ttf_parser::TtfMathFont<'a> {
    let font = ttf_parser::Face::parse(file, 0).unwrap();
    TtfMathFont::new(font).unwrap()
}