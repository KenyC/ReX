use rex::{
    render::Renderer,
    layout::{LayoutSettings, Style},
    font::{FontContext, backend::ttf_parser::TtfMathFont}, cairo::CairoBackend
};
use clap::Parser;

const DEFAULT_FONT_FILE_PATH : &str = "resources/Garamond_Math.otf";
const DEFAULT_OUTPUT_FILE : &str = "test.svg";
const DEFAULT_FORMULA: &str = &r"\iint \sqrt{1 + f^2(x,t,t)}\,\mathrm{d}x\mathrm{d}y\mathrm{d}t = \sum \xi(t)";
const DEFAULT_FONT_SIZE : f64 = 16.;

#[derive(Parser)]
struct Options {
    #[arg(default_value_t = DEFAULT_FORMULA.to_string(), help = "Formula to render")]
    formula : String,

    #[arg(short = 'i', long, conflicts_with("formula"))]
    formula_path : Option<std::path::PathBuf>,

    #[arg(short = 'o', long = "output", help = "SVG output file")]
    output_file_path : Option<std::path::PathBuf>,

    #[arg(short, long, default_value_t = false, help = "Display debug bounding boxes")]
    debug   : bool,

    #[arg(short, long = "fontfile", default_value_t = DEFAULT_FONT_FILE_PATH.to_string(), help = "Font file to use")]
    font_file_path : String,

    #[arg(short='s', long = "fontsize", default_value_t = DEFAULT_FONT_SIZE, help = "Font size (in pixels/em)")]
    font_size : f64,
}

fn main() {
    env_logger::init();
    // -- Parse command-line options
    let Options {mut formula, debug, font_file_path, formula_path, font_size, output_file_path } = Options::parse();
    let output_file_path = output_file_path.unwrap_or_else(|| DEFAULT_OUTPUT_FILE.into());
    if let Some(formula_path) = formula_path {
        formula = String::from_utf8(std::fs::read(&formula_path).unwrap()).unwrap();
    }

    // -- Load font
    let font_file = std::fs::read(font_file_path).unwrap();
    let font = load_font(&font_file);


    // -- Create ReX context
    let ctx = FontContext::new(&font);
    // 12pt = 16px
    let layout_settings = LayoutSettings::new(&ctx).font_size(font_size);



    // -- parse
    let parse_nodes = rex::parser::parse(&formula).unwrap();



    // -- layout
    let layout = rex::layout::engine::layout(&parse_nodes, layout_settings).unwrap();



    // -- create Cairo surface & context
    let dims = layout.size();
    let svg_surface = cairo::SvgSurface::new(dims.width, dims.height - dims.depth, Some(output_file_path)).unwrap();
    let context = cairo::Context::new(&svg_surface).unwrap();
    // So that top-left corner of SVG is aligned with top of formula
    context.translate(0., dims.height);



    // -- Render to Cairo backend
    let mut backend = CairoBackend::new(context);
    let mut renderer = Renderer::new();
    renderer.debug = debug;
    renderer.render(&layout, &mut backend);
    

}

fn load_font<'a>(file : &'a [u8]) -> rex::font::backend::ttf_parser::TtfMathFont<'a> {
    let font = ttf_parser::Face::parse(file, 0).unwrap();
    TtfMathFont::new(font).unwrap()
}