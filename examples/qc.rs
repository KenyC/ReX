use std::fs;
use pathfinder_export::{Export, FileFormat};
use pathfinder_renderer::scene::Scene;
use vector::{Rect, Vector};
use rex::{
    render::{Renderer, SceneWrapper},
    layout::{Grid, Layout, engine, LayoutSettings, Style},
    parser::parse,
    font::FontContext
};
use font::OpenTypeFont;

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

fn v_xy(x: f64, y: f64) -> Vector {
    Vector::new(x as f32, y as f32)
}

fn main() {
    env_logger::init();
    use font::Font;

    let samples: Vec<_> = SAMPLES.iter().cloned().map(|tex| parse(dbg!(tex)).unwrap()).collect();
    let fonts: Vec<_> = fs::read_dir("fonts").unwrap()
        .filter_map(|e| e.ok())
        .filter_map(|entry| fs::read(entry.path()).ok())
        .map(|data| OpenTypeFont::parse(&data))
        .filter(|font| font.math().is_some())
        .collect();

    let mut grid = Grid::new();
    for (row, font) in fonts.iter().enumerate() {
        let ctx = FontContext::new(&font);
        let layout_settings = LayoutSettings::new(&ctx, 10.0, Style::Display);

        for (column, sample) in samples.iter().enumerate() {
            if let Ok(node) = engine::layout(sample, layout_settings).map(|l| l.as_node()) {
                grid.insert(row, column, node);
            }
        }
    }

    let mut layout = Layout::new();
    layout.add_node(grid.build());

    let mut renderer = Renderer::new();
    let (x0, y0, x1, y1) = renderer.size(&layout);
    let mut scene = Scene::new();
    scene.set_view_box(Rect::from_points(v_xy(x0, y0), v_xy(x1, y1)));
    let mut backend = SceneWrapper::new(&mut scene);
    renderer.render(&layout, &mut backend);
    
    let mut buf = Vec::new();
    scene.export(&mut buf, FileFormat::SVG).unwrap();

    fs::write("qc.svg", &buf).unwrap();
}