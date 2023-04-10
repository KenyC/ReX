use rex::{
    render::Renderer,
    layout::{Grid, Layout, engine, LayoutSettings, Style},
    parser::parse,
    font::{FontContext, backend::ttf_parser::TtfMathFont}, raqote::RaqoteBackend
};

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
const FONT_FILE_PATH : &str = "resources/Garamond_Math.otf";


fn main() {
    env_logger::init();

    let samples: Vec<_> = SAMPLES.iter().cloned().map(|tex| parse(dbg!(tex)).unwrap()).collect();
    let font_file = std::fs::read(FONT_FILE_PATH).unwrap();
    let font = load_font(&font_file);
    let ctx = FontContext::new(&font).unwrap();

    let mut grid = Grid::new();
    
    let layout_settings = LayoutSettings::new(&ctx, 35.0, Style::Display);

    for (column, sample) in samples.iter().enumerate() {
        if let Ok(node) = engine::layout(sample, layout_settings).map(|l| l.as_node()) {
            grid.insert(column, 0, node);
        }
    }

    let mut layout = Layout::new();
    layout.add_node(grid.build());

    // Create Raqote context
    let mut context = raqote::DrawTarget::new(1000, 1000,);
    let mut backend = RaqoteBackend::new(&mut context);

    let renderer = Renderer::new();
    renderer.render(&layout, &mut backend);
    
    context.write_png("test.png").unwrap();
}

fn load_font<'a>(file : &'a [u8]) -> rex::font::backend::ttf_parser::TtfMathFont<'a> {
    let font = ttf_parser::Face::parse(file, 0).unwrap();
    TtfMathFont::new(font).unwrap()
}