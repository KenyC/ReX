use std::fs;
use pathfinder_export::{Export, FileFormat};
use pathfinder_renderer::scene::Scene;
use pathfinder_geometry::{rect::RectF, vector::vec2f};
use rex::{
    render::{Renderer, SceneWrapper},
    layout::{Grid, Layout, engine, LayoutSettings, Style},
    parser::parse,
    font::FontContext
};
use font::{Font, OpenTypeFont};

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

    let samples: Vec<_> = SAMPLES.iter().cloned().map(|tex| parse(dbg!(tex)).unwrap()).collect();
    let fonts: Vec<_> = fs::read_dir("fonts").unwrap()
        .filter_map(|e| e.ok())
        .filter_map(|entry| {
            fs::read(entry.path()).ok()
            .and_then(|data| font::parse(&data).ok().and_then(|f| f.downcast_box::<OpenTypeFont>().ok()))
            .map(|font| (font, entry.path()))
        })
        .filter(|(font, path)| font.math.is_some())
        .collect();

    let mut grid = Grid::new();
    for (row, (font, path)) in fonts.iter().enumerate() {
        let ctx = FontContext::new(font.as_ref()).unwrap();
        let layout_settings = LayoutSettings::new(&ctx, 10.0, Style::Display);

        let name = format!("\\mathtt{{{}}}", path.file_name().unwrap().to_str().unwrap());
        if let Ok(node) = engine::layout(&parse(&name).unwrap(), layout_settings).map(|l| l.as_node()) {
            grid.insert(row, 0, node);
        }
        for (column, sample) in samples.iter().enumerate() {
            if let Ok(node) = engine::layout(sample, layout_settings).map(|l| l.as_node()) {
                grid.insert(row, column+1, node);
            }
        }
    }

    let mut layout = Layout::new();
    layout.add_node(grid.build());

    let mut renderer = Renderer::new();
    let (x0, y0, x1, y1) = renderer.size(&layout);
    let mut scene = Scene::new();
    scene.set_view_box(RectF::from_points(vec2f(x0 as f32, y0 as f32), vec2f(x1 as f32, y1 as f32)));
    let mut backend = SceneWrapper::new(&mut scene);
    renderer.render(&layout, &mut backend);
    
    let mut buf = Vec::new();
    scene.export(&mut buf, FileFormat::SVG).unwrap();

    fs::write("qc.svg", &buf).unwrap();
}