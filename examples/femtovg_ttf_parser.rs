extern crate sdl2; 
extern crate femtovg; 
 
use std::fs;
use std::path::PathBuf;

use femtovg::Color;
use femtovg::renderer::OpenGl;
use rex::femtovg::FemtoVGCanvas;
use rex::font::{MathFont, FontContext};
use rex::font::backend::ttf_parser::MathFont;
use rex::{Renderer, Backend};
use rex::layout::Layout;
use rex::parser::parse;
use sdl2::rect::Rect; 
use sdl2::event::Event; 
use sdl2::keyboard::Keycode; 
use sdl2::render::Canvas;
// use sdl2::render::Canvas; 
use sdl2::video::Window; 

const SAMPLES: &[&str] = &[
    r"\iint \sqrt{1 + f^2(x,t,t)}\,\mathrm{d}x\mathrm{d}y\mathrm{d}t = \sum \xi(t)",
    r"\Vert f \Vert_2 = \sqrt{\int f^2(x)\,\mathrm{d}x}",
    r"\left.x^{x^{x^x_x}_{x^x_x}}_{x^{x^x_x}_{x^x_x}}\right\} \mathrm{wat?}",
    r"\hat A\grave A\bar A\tilde A\hat x \grave x\bar x\tilde x\hat y\grave y\bar y\tilde y",
    r"\mathop{\overbrace{1+2+3+\unicodecdots+n}}\limits^{\mathrm{Arithmatic}} = \frac{n(n+1)}{2}",
    r"\sigma = \left(\int f^2(x)\,\mathrm{d}x\right)^{1/2}",
    r"\left\vert\sum_k a_k b_k\right\vert \leq \left(\sum_k a_k^2\right)^{\frac12}\left(\sum_k b_k^2\right)^{\frac12}",
    r"f^{(n)}(z) = \frac{n!}{2\pi i} \oint \frac{f(\xi)}{(\xi - z)^{n+1}}\,\mathrm{d}\xi",
    r"\frac{1}{\left(\sqrt{\phi\sqrt5} - \phi\right) e^{\frac{2}{5}\pi}} = 1 + \frac{e^{-2\pi}}{1 + \frac{e^{-4\pi}}{1 + \frac{e^{-6\pi}}{1 + \frac{e^{-8\pi}}{1 + \unicodecdots}}}}",
    r"\mathop{\mathrm{lim\,sup}}\limits_{x\rightarrow\infty}\ \mathop{\mathrm{sin}}(x)\mathrel{\mathop{=}\limits^?}1"
];
 
const WIDTH : u32 = 800; 
const HEIGHT: u32 = 600; 

fn main() { 
    env_logger::init();

    let sdl_context = sdl2::init().unwrap(); 
    let video_subsystem = sdl_context.video().unwrap(); 
 

    let gl_attr = video_subsystem.gl_attr();
    gl_attr.set_context_profile(sdl2::video::GLProfile::Core);
    gl_attr.set_stencil_size(1);
    gl_attr.set_multisample_samples(4);
    gl_attr.set_context_version(4, 2);
 
 
    let window = video_subsystem.window("ReX and femtovg", WIDTH, HEIGHT) 
        .position_centered() 
        .opengl() 
        .build() 
        .unwrap(); 
 

    let _gl_context = window.gl_create_context().unwrap();
    let renderer = unsafe{OpenGl::new_from_function(|string| video_subsystem.gl_get_proc_address(string).cast())}.expect("Cannot create renderer");
    let mut canvas = femtovg::Canvas::new(renderer).expect("Cannot create canvas");
    let (_, dpi_factor, _) = video_subsystem.display_dpi(0).expect("Can't get a dpi");
    canvas.set_size(WIDTH, HEIGHT, dpi_factor);
    // canvas.scale(3., 3.);

    const WHITE : femtovg::Color = femtovg::Color::white(); 
    const BLACK : femtovg::Color = femtovg::Color::black(); 
    let paint = femtovg::Paint::color(BLACK).with_fill_rule(femtovg::FillRule::EvenOdd).with_anti_alias(true);
    let font_file = std::fs::read("fonts/rex-xits_old.otf").unwrap();
    let font = load_font(&font_file);

 
    let mut event_pump = sdl_context.event_pump().unwrap(); 
    'running: loop { 
        canvas.clear_rect(0, 0, WIDTH, HEIGHT, WHITE);
        let mut canvas_backend = FemtoVGCanvas::new(&mut canvas, paint.clone());
        draw(&mut canvas_backend, &font);
        // draw_simple(&mut canvas_backend, &fonts);
        canvas.flush();
        window.gl_swap_window();
        let event_iter = std::iter::once(event_pump.wait_event()).chain(event_pump.poll_iter());
        for event in event_iter { 
            match event { 
                Event::Quit { .. } | 
                Event::KeyDown { keycode: Some(Keycode::Escape), .. } => { 
                    break 'running 
                }, 
                _ => {} 
            } 
        } 
    } 
}


fn draw<'a, 'b : 'a>(backend : &'b mut FemtoVGCanvas<'a, OpenGl>, fonts : &MathFont<'a>) {
    let samples: Vec<_> = SAMPLES.iter().cloned().map(|tex| parse(dbg!(tex)).unwrap()).collect();

    let mut grid = rex::layout::Grid::new();
    let font_context = FontContext::new(fonts).unwrap();
    let layout_settings = rex::layout::LayoutSettings::new(&font_context, 10.0, rex::layout::Style::Display);

    for (column, sample) in samples.iter().enumerate() {
        if let Ok(node) = rex::layout::engine::layout(sample, layout_settings).map(|l| l.as_node()) {
            grid.insert(0, column+1, node);
        }
    }
    let mut layout = Layout::new();
    layout.add_node(grid.build());

    let renderer = Renderer::new();

    let (x0, y0, x1, y1) = renderer.size(&layout);
    let width  = (x1 - x0) as f32;
    let height = (y1 - y0) as f32;
    // {
    let canvas = backend.canvas();
    canvas.save();
    canvas.translate(x0 as f32, y0 as f32);
    let min_scale = (WIDTH as f32) / width;
    canvas.scale(min_scale, min_scale);
    println!("{:?}", min_scale);
    // if min_scale <= 1. {
    // }
    // canvas.scale(4.2, 4.2);
    // }


    // let glyph_id = fonts.glyph_index('a').unwrap();
    // backend.symbol(rex::Cursor { x: 50., y: 50. }, glyph_id.0 as u16, 10., fonts);
    // backend.symbol(rex::Cursor { x: 230.11639868416952, y: 59.99500284960959,}, 42, 10., fonts);
    renderer.render(&layout, backend);

    let canvas = backend.canvas();
    canvas.restore()
}

fn load_font<'a>(file : &'a [u8]) -> MathFont<'a> {
    let font = ttf_parser::Face::parse(file, 0).unwrap();
    MathFont::new(font).unwrap()
}