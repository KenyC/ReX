extern crate sdl2; 
extern crate femtovg; 
 


use rex::layout::engine::{LayoutBuilder, LayoutEngine};
use sdl2::event::Event; 
use sdl2::keyboard::Keycode; 
use femtovg::renderer::OpenGl;

use rex::femtovg::FemtoVGCanvas;
use rex::font::backend::ttf_parser::TtfMathFont;
use rex::Renderer;
use rex::layout::{Grid, Layout, LayoutBBox};

const SAMPLES: &[&str] = &[
    r"\iint \sqrt{1 + f^2(x',t'',t''')}\,\mathrm{d}x\mathrm{d}y\mathrm{d}t = \sum \xi(t)",
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
 
const WIDTH :    u32 = 800; 
const HEIGHT:    u32 = 600; 
const FONT_SIZE: f64 = 10.;

fn main() { 
    env_logger::init();


    // -- Load font
    let font_file = std::fs::read("resources/FiraMath_Regular.otf").unwrap();
    let font = load_font(&font_file);

    let layout_engine = LayoutBuilder::new(&font).font_size(FONT_SIZE).build();



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
    draw(&mut canvas_backend, &layout_engine, &SAMPLES);

    canvas.flush();
    window.gl_swap_window();






    // -- Wait calmly for exit
    wait_for_exit(sdl_context); 
}


fn draw<'a, 'b : 'a>(backend : &'b mut FemtoVGCanvas<'a, OpenGl>, layout_engine : &LayoutEngine<'_, TtfMathFont>, formulas : &[&str]) 
{
    // -- Parse formula
    let sample_parse_nodes = 
    	formulas
    	.into_iter()
    	.map(|f| rex::parser::parse(f).unwrap())
    ;


    // -- Lay out nodes in grid pattern
    // TODO: expose VBox and HBox
    let mut grid = Grid::new();
    for (i, parse_nodes) in sample_parse_nodes.into_iter().enumerate() {
    	let layout = layout_engine.layout(&parse_nodes).unwrap();
    	grid.insert(i, 0, layout.as_node());
    }
    let mut layout = Layout::new();
    layout.add_node(grid.build());



    // -- Transform canvas to lay out formulas
    let bbox = layout.full_bounding_box();
    center_formula(bbox, backend);


    // -- Render
    let renderer = Renderer::new();
    renderer.render(&layout, backend);

    backend.canvas().restore();
}






fn center_formula(bbox: LayoutBBox, backend: &mut FemtoVGCanvas<OpenGl>) {
    let width   = bbox.width()  as f32;
    let height  = bbox.height() as f32;

    let width_canvas  = WIDTH as f32;
    let height_canvas = HEIGHT as f32;

    let canvas = backend.canvas();
    canvas.save();
    canvas.translate(width_canvas / 2., height_canvas / 2.);

    let min_width_scale  = width_canvas / width;
    let min_height_scale = height_canvas / height;
    let min_scale = f32::min(min_width_scale, min_height_scale);
    // canvas.scale(10., 10.);
    canvas.scale(min_scale, min_scale);

    // Translating to origin
    canvas.translate(
        (- (bbox.x_min + bbox.x_max) * 0.5) as f32, 
        (- (bbox.y_min + bbox.y_max) * 0.5) as f32,
    );

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