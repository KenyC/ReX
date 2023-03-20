extern crate sdl2; 
extern crate femtovg; 
 
use std::fs;
use std::path::PathBuf;

use femtovg::Color;
use femtovg::renderer::OpenGl;
use font::OpenTypeFont;
use rex::femtovg::FemtoVGCanvas;
use rex::{Renderer, Backend};
use rex::layout::Layout;
use rex::parser::parse;
use sdl2::rect::Rect; 
use sdl2::event::Event; 
use sdl2::keyboard::Keycode; 
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
 
fn main() { 
    env_logger::init();

    let sdl_context = sdl2::init().unwrap(); 
    let video_subsystem = sdl_context.video().unwrap(); 
 
    let width = 800; 
    let height = 600; 

    let gl_attr = video_subsystem.gl_attr();
    gl_attr.set_context_profile(sdl2::video::GLProfile::Core);
    gl_attr.set_stencil_size(1);
    gl_attr.set_multisample_samples(4);
    gl_attr.set_context_version(4, 2);
 
 
    let window = video_subsystem.window("ReX and femtovg", width, height) 
        .position_centered() 
        .opengl() 
        .build() 
        .unwrap(); 
 

    let _gl_context = window.gl_create_context().unwrap();
    let renderer = unsafe{OpenGl::new_from_function(|string| video_subsystem.gl_get_proc_address(string).cast())}.expect("Cannot create renderer");
    let mut canvas = femtovg::Canvas::new(renderer).expect("Cannot create canvas");
    let (_, dpi_factor, _) = video_subsystem.display_dpi(0).expect("Can't get a dpi");
    canvas.set_size(width, height, dpi_factor);
    // canvas.scale(3., 3.);

    const WHITE : femtovg::Color = femtovg::Color::white(); 
    const BLACK : femtovg::Color = femtovg::Color::black(); 
    let paint = femtovg::Paint::color(BLACK).with_fill_rule(femtovg::FillRule::EvenOdd).with_anti_alias(true);
    let fonts = load_fonts();

 
    let mut event_pump = sdl_context.event_pump().unwrap(); 
    'running: loop { 
        canvas.clear_rect(0, 0, width, height, WHITE);
        let mut canvas_backend = FemtoVGCanvas::new(&mut canvas, paint.clone());
        draw(&mut canvas_backend, &fonts);
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

fn draw_simple<B : Backend<OpenTypeFont>>(backend : &mut B, fonts : &[(Box<OpenTypeFont>, PathBuf)]) {
    // let samples: Vec<_> = SAMPLES.iter().cloned().map(|tex| parse(dbg!(tex)).unwrap()).collect();
    // let samples: Vec<_> = SAMPLES.iter().cloned().map(|tex| parse(tex).unwrap()).collect();

    let mut grid = rex::layout::Grid::new();

    if let Some((font, path)) = fonts.first() {
        let ctx = rex::font::FontContext::new(font.as_ref()).unwrap();
        let layout_settings = rex::layout::LayoutSettings::new(&ctx, 10.0, rex::layout::Style::Display);

        let layout = rex::layout::engine::layout(&parse("ab").unwrap(), layout_settings);
        if let Ok(l) = layout {
            let node = l.as_node();
            grid.insert(0, 0, node);
        }
    }

    let mut layout = Layout::new();
    layout.add_node(grid.build());

    let mut renderer = Renderer::new();
    renderer.render(&layout, backend);
}

fn draw<B : Backend<OpenTypeFont>>(backend : &mut B, fonts : &[(Box<OpenTypeFont>, PathBuf)]) {

    let samples: Vec<_> = SAMPLES.iter().cloned().map(|tex| parse(dbg!(tex)).unwrap()).collect();

    let mut grid = rex::layout::Grid::new();
    for (row, (font, path)) in fonts.iter().enumerate() {
        let ctx = rex::font::FontContext::new(font.as_ref()).unwrap();
        let layout_settings = rex::layout::LayoutSettings::new(&ctx, 10.0, rex::layout::Style::Display);

        let name = format!("\\mathtt{{{}}}", path.file_name().unwrap().to_str().unwrap());
        if let Ok(node) = rex::layout::engine::layout(&parse(&name).unwrap(), layout_settings).map(|l| l.as_node()) {
            grid.insert(row, 0, node);
        }
        for (column, sample) in samples.iter().enumerate() {
            if let Ok(node) = rex::layout::engine::layout(sample, layout_settings).map(|l| l.as_node()) {
                grid.insert(row, column+1, node);
            }
        }
    }

    let mut layout = Layout::new();
    layout.add_node(grid.build());

    let renderer = Renderer::new();
    renderer.render(&layout, backend);
}

fn load_fonts() -> Vec<(Box<OpenTypeFont>, PathBuf)> {
    fs::read_dir("fonts").unwrap()
        .filter_map(|e| e.ok())
        .filter_map(|entry| {
            fs::read(entry.path()).ok()
            .and_then(|data| font::parse(&data).ok().and_then(|f| f.downcast_box::<OpenTypeFont>().ok()))
            .map(|font| (font, entry.path()))
        })
        .filter(|(font, path)| font.math.is_some())
    .collect()
}