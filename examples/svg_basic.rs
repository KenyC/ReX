use rex::{
    cairo::CairoBackend, font::backend::ttf_parser::TtfMathFont, layout::engine::LayoutBuilder, render::Renderer
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
    // 12pt = 16px
    let layout_engine = LayoutBuilder::new(&font)
        .font_size(font_size)
        .build();


    // -- parse
    let parse_nodes = rex::parser::parse(&formula).unwrap();

    if debug {
        eprintln!("{:#?}", parse_nodes);
    }


    // -- layout
    let layout = layout_engine.layout(&parse_nodes).unwrap();

    if debug {
        eprintln!("{:#?}", layout.clone().as_node());
    }

    // -- create Cairo surface & context
    let bbox = layout.full_bounding_box();
    let svg_surface = cairo::SvgSurface::new(bbox.width(), bbox.height(), Some(output_file_path)).unwrap();
    let context = cairo::Context::new(&svg_surface).unwrap();
    // Translate to origin
    context.translate(- bbox.x_min, - bbox.y_min);



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