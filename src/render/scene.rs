
use pathfinder_renderer::{
    scene::{Scene, DrawPath},
    paint::{Paint, PaintId},
};
use pathfinder_content::{
    outline::{Outline},
    stroke::{StrokeStyle, LineCap, LineJoin, OutlineStrokeToFill},
};
use pathfinder_geometry::{
    transform2d::Transform2F,
    vector::Vector2F,
    rect::RectF,
};
use pathfinder_color::ColorU;
use super::{Backend, Cursor, Role};
use crate::{font::MathFont, error::FontError};
use crate::parser::{color::RGBA};

fn v_cursor(c: Cursor) -> Vector2F {
    Vector2F::new(c.x as f32, c.y as f32)
}
fn v_xy(x: f64, y: f64) -> Vector2F {
    Vector2F::new(x as f32, y as f32)
}

pub struct SceneWrapper<'a> {
    scene: &'a mut Scene,
    color_stack: Vec<PaintId>,
    transform: Transform2F,
    paint: PaintId
}
impl<'a> SceneWrapper<'a> {
    pub fn new(scene: &'a mut Scene) -> Self {
        SceneWrapper::with_transform(scene, Transform2F::default())
    }
    pub fn with_transform(scene: &'a mut Scene, transform: Transform2F) -> Self {
        SceneWrapper {
            paint: scene.push_paint(&Paint::black()),
            scene,
            color_stack: Vec::new(),
            transform
        }
    }
}

impl<'a> Backend for SceneWrapper<'a> {
    fn bbox(&mut self, pos: Cursor, width: f64, height: f64, role: Role) {
        let color = match role {
            Role::Glyph => ColorU::new(0, 200, 0, 255),
            Role::HBox => ColorU::new(200, 0, 0, 255),
            Role::VBox => ColorU::new(0, 0, 200, 255),
        };
        let paint = self.scene.push_paint(&Paint::from_color(color));
        let style = StrokeStyle {
            line_cap: LineCap::Square,
            line_join: LineJoin::Bevel,
            line_width: 0.1
        };
        let outline = Outline::from_rect(RectF::new(v_cursor(pos), v_xy(width, height)));
        let mut stroke = OutlineStrokeToFill::new(&outline, style);
        stroke.offset();
        let outline = stroke.into_outline().transformed(&self.transform);
        self.scene.push_draw_path(DrawPath::new(outline, paint));
    }
    fn symbol(&mut self, pos: Cursor, gid: u16, scale: f64, font: &MathFont) {
        use font::{Font, GlyphId};
        let path = font.glyph(GlyphId(gid as u32)).unwrap().path;
        let tr = self.transform
            * Transform2F::from_translation(v_cursor(pos))
            * Transform2F::from_scale(v_xy(scale, -scale))
            * font.font_matrix();
        
        self.scene.push_draw_path(DrawPath::new(path.transformed(&tr), self.paint));
    }
    fn rule(&mut self, pos: Cursor, width: f64, height: f64) {
        let origin = v_cursor(pos);
        let size = v_xy(width, height);

        let outline = Outline::from_rect(RectF::new(origin, size));
        self.scene.push_draw_path(DrawPath::new(outline.transformed(&self.transform), self.paint));
    }
    fn begin_color(&mut self, RGBA(r, g, b, a): RGBA) {
        self.color_stack.push(self.paint);
        self.paint = self.scene.push_paint(&Paint::from_color(ColorU::new(r, g, b, a)));
    }
    fn end_color(&mut self) {
        self.paint = self.color_stack.pop().unwrap();
    }
}

use super::{Renderer};
use crate::font::FontContext;
use crate::layout::{LayoutSettings, Style};
use pathfinder_export::{Export, FileFormat};

pub fn svg(font: &[u8], tex: &str) -> Result<Vec<u8>, FontError> {
    // TODO : remove '.unwrap()' 
    let font = MathFont::parse(font).unwrap();
    let ctx = FontContext::new(&font)?;
    let mut renderer = Renderer::new();
    renderer.debug = true;
    let layout_settings = LayoutSettings::new(&ctx, 10.0, Style::Display);
    let layout = renderer.layout(tex, layout_settings).unwrap();
    let (x0, y0, x1, y1) = renderer.size(&layout);
    let mut scene = Scene::new();
    scene.set_view_box(RectF::from_points(v_xy(x0, y0), v_xy(x1, y1)));
    let mut backend = SceneWrapper::new(&mut scene);
    renderer.render(&layout, &mut backend);

    let mut buf = Vec::new();
    scene.export(&mut buf, FileFormat::SVG).unwrap();
    Ok(buf)
}