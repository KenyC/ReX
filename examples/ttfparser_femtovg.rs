extern crate sdl2; 
extern crate femtovg; 
 





use femtovg::renderer::OpenGl;
use rex::femtovg::FemtoVGCanvas;
use rex::font::FontContext;
use rex::font::backend::ttf_parser::TtfMathFont;
use rex::Renderer;
use rex::layout::Layout;
use rex::parser::parse;
use sdl2::event::Event; 
use sdl2::keyboard::Keycode; 

const SAMPLES: &[&str] = &[
    r"\left\{\begin{array}{c}1\\2\\3\\4\\5\\6\\7\\5\\6\\7\\5\\6\\7\\5\\6\\7\\5\\6\\7\\5\\6\\7\end{array}\right\}",
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
    // let font_file = std::fs::read("fonts/rex-xits_old.otf").unwrap();
    let font_file = std::fs::read("resources/FiraMath_Regular.otf").unwrap();
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


fn draw<'a, 'b : 'a>(backend : &'b mut FemtoVGCanvas<'a, OpenGl>, fonts : &TtfMathFont<'a>) {
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

    let dims = layout.size();
    let width  = dims.width as f32;
    let height = (dims.height - dims.depth) as f32;
    // {
    let canvas = backend.canvas();
    canvas.save();
    canvas.translate(0_f32, dims.depth as f32);
    let min_width_scale  = (WIDTH  as f32) / width;
    let min_height_scale = (HEIGHT as f32) / height;
    let min_scale = f32::min(min_width_scale, min_height_scale) * 1.00;
    canvas.scale(min_scale, min_scale);


    // let glyph_id = fonts.glyph_index('a').unwrap();
    // backend.symbol(rex::Cursor { x: 50., y: 50. }, glyph_id.0 as u16, 10., fonts);
    // backend.symbol(rex::Cursor { x: 230.11639868416952, y: 59.99500284960959,}, 42, 10., fonts);
    renderer.render(&layout, backend);

    let canvas = backend.canvas();
    canvas.restore()
}

fn load_font<'a>(file : &'a [u8]) -> rex::font::backend::ttf_parser::TtfMathFont<'a> {
    let font = ttf_parser::Face::parse(file, 0).unwrap();
    TtfMathFont::new(font).unwrap()
}