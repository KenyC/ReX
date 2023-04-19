use rex::{
    render::Renderer,
    layout::{LayoutSettings, Style},
    font::{FontContext, backend::ttf_parser::TtfMathFont}, cairo::CairoBackend
};

const FONT_FILE_PATH : &str = "resources/Garamond_Math.otf";
const DEFAULT_FORMULA: &str = &r"\iint \sqrt{1 + f^2(x,t,t)}\,\mathrm{d}x\mathrm{d}y\mathrm{d}t = \sum \xi(t)";



fn main() {
    env_logger::init();
    let formula = std::env::args().nth(1).unwrap_or(DEFAULT_FORMULA.to_string());


    // -- Load font
    let font_file = std::fs::read(FONT_FILE_PATH).unwrap();
    let font = load_font(&font_file);


    // -- Create ReX context
    let ctx = FontContext::new(&font).unwrap();
    let layout_settings = LayoutSettings::new(&ctx, 10.0, Style::Display);



    // -- parse
    let parse_nodes = rex::parser::parse(&formula).unwrap();



    // -- layout
    let layout = rex::layout::engine::layout(&parse_nodes, layout_settings).unwrap();



    // -- create Cairo surface & context
    let dims = layout.size();
    let svg_surface = cairo::SvgSurface::new(dims.width, dims.height - dims.depth, Some("test.svg")).unwrap();
    let context = cairo::Context::new(&svg_surface).unwrap();
    // So that top-left corner of SVG is aligned with top of formula
    context.translate(0., dims.height);



    // -- Render to Cairo backend
    let mut backend = CairoBackend::new(context);
    let renderer = Renderer::new();
    renderer.render(&layout, &mut backend);
    

}

fn load_font<'a>(file : &'a [u8]) -> rex::font::backend::ttf_parser::TtfMathFont<'a> {
    let font = ttf_parser::Face::parse(file, 0).unwrap();
    TtfMathFont::new(font).unwrap()
}