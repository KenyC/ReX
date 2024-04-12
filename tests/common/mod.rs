#![allow(dead_code)]
pub mod debug_render;
pub mod svg_diff;
pub mod svg;

use std::path::Path;
use self::debug_render::Equation;
use bincode;
use rex::{font::{FontContext, backend::ttf_parser::TtfMathFont}, parser::parse, layout::{LayoutSettings, Style, Grid}, Renderer, cairo::CairoBackend};

pub fn load_bincode<P: AsRef<Path>>(path: P) -> Vec<Equation> {
    use std::fs::File;
    use std::io::BufReader;

    let file = File::open(path.as_ref()).expect("failed to open test collection");
    let mut reader = BufReader::new(file);
    let tests: Vec<Equation> = bincode::deserialize_from(&mut reader)
        .expect("failed to load historical test results");

    tests
}

pub fn equation_diffs(old: &[Equation], new: &[Equation]) -> Result<Vec<(Equation, Equation)>, ()> {
    if old.len() != new.len() {
        return Err(());
    }

    let mut diff: Vec<(Equation, Equation)> = Vec::new();
    for (left, right) in Iterator::zip(old.iter(), new.iter()) {
        if left != right {
            diff.push((left.clone(), right.clone()));
        }
    }

    Ok(diff)
}

pub fn render<'a, 'f, 'b,>(ctx : &FontContext<'f, TtfMathFont<'a>>, string : &'b str,) -> Result<Equation, rex::error::Error<'b>> {
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
    let svg_surface = cairo::ImageSurface::create(
        cairo::Format::ARgb32, 
        formula_bbox.width.ceil() as i32, 
        (formula_bbox.height - formula_bbox.depth).ceil() as i32,
    ).unwrap();
    let context = cairo::Context::new(&svg_surface).unwrap();
    let mut backend = CairoBackend::new(context);
    renderer.render(&layout, &mut backend);
    // svg_surface.write_to_png(stream);

    todo!()
    // let equation = Equation {
    //     tex:    equation.to_string(),
    //     width:  renderer.width.take(),
    //     height: renderer.height.take(),
    //     render: canvas,
    //     description,
    // };

    // Ok(())
}


pub fn load_font<'a>(file : &'a [u8]) -> rex::font::backend::ttf_parser::TtfMathFont<'a> {
    let font = ttf_parser::Face::parse(file, 0).unwrap();
    TtfMathFont::new(font).unwrap()
}