extern crate sdl2; 
extern crate femtovg; 
 


use rex::layout::engine::layout;
use sdl2::event::Event; 
use sdl2::keyboard::Keycode; 
use femtovg::renderer::OpenGl;

use rex::femtovg::FemtoVGCanvas;
use rex::font::FontContext;
use rex::font::backend::ttf_parser::TtfMathFont;
use rex::Renderer;
use rex::layout::{LayoutDimensions};

const DEFAULT_FORMULA: &str = &r"\iint \sqrt{1 + f^2(x,t,t)}\,\mathrm{d}x\mathrm{d}y\mathrm{d}t = \sum \xi(t)";
 
const WIDTH : u32 = 800; 
const HEIGHT: u32 = 600; 

fn main() { 
    env_logger::init();
    let formula = std::env::args().nth(1).unwrap_or(DEFAULT_FORMULA.to_string());


    // -- Load font
    let font_file = std::fs::read("resources/FiraMath_Regular.otf").unwrap();
    let font = load_font(&font_file);



    // -- Init SDL and OpenGL
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
 




    // -- Init femtovg
    let renderer = unsafe{OpenGl::new_from_function(|string| video_subsystem.gl_get_proc_address(string).cast())}.expect("Cannot create renderer");
    let mut canvas = femtovg::Canvas::new(renderer).expect("Cannot create canvas");
    let (_, dpi_factor, _) = video_subsystem.display_dpi(0).expect("Can't get a dpi");
    canvas.set_size(WIDTH, HEIGHT, dpi_factor);



    // -- Render
    window.gl_swap_window();
    canvas.clear_rect(0, 0, WIDTH, HEIGHT, femtovg::Color::white());

    // -- Create backend with initial black paint
    let paint = 
        femtovg::Paint::color(femtovg::Color::black())
        .with_fill_rule(femtovg::FillRule::EvenOdd)
        .with_anti_alias(true)
    ;
    let mut canvas_backend = FemtoVGCanvas::new(&mut canvas, paint.clone());

    // -- Draw
    // Calls to ReX function are limited to this function
    draw(&mut canvas_backend, &font, &formula);

    canvas.flush();
    window.gl_swap_window();






    // -- Wait calmly for exit
    wait_for_exit(sdl_context); 
}


fn draw<'a, 'b : 'a>(backend : &'b mut FemtoVGCanvas<'a, OpenGl>, font : &TtfMathFont<'a>, formula : &str) 
{
    // -- Create context
    let font_context = FontContext::new(font).unwrap();
    let layout_settings = rex::layout::LayoutSettings::new(&font_context, 10.0, rex::layout::Style::Display);


    // -- Parse formula
    let parse_nodes = rex::parser::parse(formula).unwrap();


    // -- Lay out nodes
    let layout = layout(&parse_nodes, layout_settings).unwrap();


    // -- Transform canvas to lay out formulas
    let dims = layout.size();
    center_formula(dims, backend);


    // -- Render
    let renderer = Renderer::new();
    renderer.render(&layout, backend);

    backend.canvas().restore();
}






fn center_formula(dims: LayoutDimensions, backend: &mut FemtoVGCanvas<OpenGl>) {
    let width   = dims.width  as f32;
    let height  = dims.height as f32;
    let depth   = dims.depth  as f32;
    let width_canvas  = WIDTH as f32;
    let height_canvas = HEIGHT as f32;

    let canvas = backend.canvas();
    canvas.save();
    canvas.translate(width_canvas / 2., height_canvas / 2.);

    let min_width_scale  = width_canvas / width;
    let min_height_scale = (height_canvas / 2.) / height.abs();
    let min_depth_scale = (height_canvas / 2.) / depth.abs();
    let min_scale = f32::min(min_width_scale, f32::min(min_height_scale, min_depth_scale));
    // canvas.scale(10., 10.);
    canvas.scale(min_scale, min_scale);
    canvas.translate(- width / 2., 0.);

}




fn wait_for_exit(sdl_context: sdl2::Sdl) {
    let mut event_pump = sdl_context.event_pump().unwrap();
    'running: loop { 
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



fn load_font<'a>(file : &'a [u8]) -> rex::font::backend::ttf_parser::TtfMathFont<'a> {
    let font = ttf_parser::Face::parse(file, 0).unwrap();
    TtfMathFont::new(font).unwrap()
}