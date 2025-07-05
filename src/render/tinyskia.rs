//! Provides a [`Backend`] for tiny-skia
//!
//! This allows to render onto a canvas of RGBA pixels ([`pixmap`]),
//! which can then be used inside a [`tiny_skia`] application,
//! or convert to a PNG image.

use super::{Backend, Cursor, Role};
use crate::layout::LayoutDimensions;
use crate::parser::color::RGBA;
use crate::{font::common::GlyphId, FontBackend, GraphicsBackend};
use tiny_skia::{Color, FillRule, Paint, PathBuilder, Pixmap, Rect, Stroke, Transform};
#[cfg(feature="ttfparser-fontparser")]
use crate::font::backend::ttf_parser::TtfMathFont;
#[cfg(feature="fontrs-fontparser")]
use font::{Font, OpenTypeFont};
#[cfg(feature="fontrs-fontparser")]
use pathfinder_content::{outline::ContourIterFlags, segment::SegmentKind};

/// Backend for TinySkia renderer
pub struct TinySkiaBackend {
    /// A canvas to draw onto
    pixmap: Pixmap,
    /// Transform to convert from position according to ReX Renderer backend
    /// to coordinates on the TinySkia pixmap
    pub layout_to_pixmap: Transform,
    color_stack: Vec<Color>,
    current_color: Color,
    paint: Paint<'static>,
}

impl TinySkiaBackend {
    /// New TinySkiaBackend instance taking ownership of a pixmap to draw onto.
    /// The `layout_to_pixmap` transform is initialized to the identity
    /// (resulting in everything above the baseline of the equation being
    /// clipped off the top pixmap and the left side aligned with the pixmap).
    /// Make sure to adjust that transform to position the equation appropriately.
    pub fn new(pixmap: Pixmap) -> Self {
        let layout_to_pixmap = Transform::identity();

        let current_color = Color::BLACK;
        let mut paint = Paint::default();
        paint.set_color(current_color);

        Self { pixmap, layout_to_pixmap, color_stack: vec![], current_color, paint }
    }

    /// New TinySkiaBackend instance initializing a canvas (pixmap)
    /// along with a transform to position the equation to fit the pixmap.
    /// Size in pixels can be adjusted with the scale parameter.
    pub fn from_dims(dims: LayoutDimensions, scale: f64) -> Option<Self> {
        // `Cursor` positions in layout from ReX Renderer backend are relative to the baseline,
        // including negative y-coordinates above the baseline.
        // Coordinates on `pixmap` are relative to the top-left corner of the pixmap
        // (and always positive).
        let width = (dims.width * scale) as u32;
        let height = ((dims.height - dims.depth) * scale) as u32;
        let pixmap = Pixmap::new(width, height)?;

        let scale = scale as f32;
        let layout_to_pixmap = Transform::from_translate(0.0, dims.height as f32)
            .post_scale(scale, scale);

        let current_color = Color::BLACK;
        let mut paint = Paint::default();
        paint.set_color(current_color);

        Some(Self { pixmap, layout_to_pixmap, color_stack: vec![], current_color, paint })
    }

    /// Specify initial colour to use for drawing
    pub fn set_color(&mut self, color: Color) {
        self.current_color = color;
        self.paint.set_color(color);
    }

    /// Returns pixmap being drawn onto after all drawing operations are completed
    pub fn pixmap(self) -> Pixmap {
        self.pixmap
    }
}

#[cfg(feature="ttfparser-fontparser")]
impl FontBackend<TtfMathFont<'_>> for TinySkiaBackend {
    fn symbol(&mut self, pos: Cursor, gid: GlyphId, scale: f64, ctx: &TtfMathFont<'_>) {
        // Make the tiny_skia path builder implement the necessary trait to draw
        // the glyph with the TtfMathFont font backend
        struct Builder {
            open_path: tiny_skia::PathBuilder,
        }

        impl ttf_parser::OutlineBuilder for Builder {
            fn move_to(&mut self, x: f32, y: f32) {
                self.open_path.move_to(x, y);
            }
            fn line_to(&mut self, x: f32, y: f32) {
                self.open_path.line_to(x, y);
            }
            fn quad_to(&mut self, x1: f32, y1: f32, x: f32, y: f32) {
                self.open_path.quad_to(x1, y1, x, y);
            }
            fn curve_to(&mut self, x1: f32, y1: f32, x2: f32, y2: f32, x: f32, y: f32) {
                self.open_path.cubic_to(x1, y1, x2, y2, x, y);
            }
            fn close(&mut self) {
                self.open_path.close();
            }
        }

        // Define transform to map the glyph from the position according to the font,
        // to the correct position on the tiny_skia pixmap.
        let ttf_parser::cff::Matrix { sx, ky, kx, sy, tx, ty } = ctx.font_matrix();
        let transform = Transform::from_row(sx, ky, kx, sy, tx, ty)
            // Glyph scaling and vertical flip translating
            // between font convention and ReX Renderer convention
            .post_scale(scale as f32, -scale as f32)
            // Translate to position according to ReX Renderer backend
            .post_translate(pos.x as f32, pos.y as f32)
            // Transform to coordinates on pixmap
            .post_concat(self.layout_to_pixmap);

        let mut builder = Builder { open_path: PathBuilder::new() };
        ctx.font().outline_glyph(gid.into(), &mut builder);
        if let Some(path) =  builder.open_path.finish() {
            self.pixmap.fill_path(
                &path,
                &self.paint,
                FillRule::Winding,
                transform,
                None,
            );
        }
    }
}

#[cfg(feature="fontrs-fontparser")]
impl FontBackend<OpenTypeFont> for TinySkiaBackend {
    fn symbol(&mut self, pos: Cursor, gid: GlyphId, scale: f64, ctx: &OpenTypeFont) {
        // Create tiny_skia path for glyph taking into account font matrix
        let mut contour_path = PathBuilder::new();
        {
            let tr = ctx.font_matrix();
            let path = ctx.glyph(gid.into()).unwrap().path.transformed(&tr);
            let contours = path.into_contours();
            for contour in contours {
                if let Some(segment) = contour.iter(ContourIterFlags::empty()).next() {
                    let baseline = segment.baseline;
                    contour_path.move_to(baseline.from_x(), baseline.from_y());
                }

                for segment in contour.iter(ContourIterFlags::empty()) {
                    let baseline = segment.baseline;
                    let control  = segment.ctrl;
                    match segment.kind {
                        SegmentKind::None => (),
                        SegmentKind::Line => {
                            contour_path.line_to(baseline.to_x(),   baseline.to_y());
                        },
                        SegmentKind::Quadratic => {
                            contour_path.quad_to(
                                control.from_x(),  control.from_y(),
                                baseline.to_x(), baseline.to_y()
                            );
                        },
                        SegmentKind::Cubic => {
                            contour_path.cubic_to(
                                control.from_x(),  control.from_y(),
                                control.to_x(),    control.to_y(),
                                baseline.to_x(),   baseline.to_y()
                            );
                        },
                    }
                }
            }
        }

        // Draw and fill glyph path onto pixmap with correct scaling and position
        let transform =
            // Glyph scaling and vertical flip translating
            // between font convention and ReX Renderer convention
            Transform::from_scale(scale as f32, -scale as f32)
            // Translate to position according to ReX Renderer backend
            .post_translate(pos.x as f32, pos.y as f32)
            // Transform to coordinates on pixmap
            .post_concat(self.layout_to_pixmap);
        if let Some(path) =  contour_path.finish() {
            self.pixmap.fill_path(
                &path,
                &self.paint,
                FillRule::Winding,
                transform,
                None,
            );
        }
    }
}

#[cfg(feature="ttfparser-fontparser")]
impl Backend<TtfMathFont<'_>> for TinySkiaBackend {}

#[cfg(feature="fontrs-fontparser")]
impl Backend<OpenTypeFont> for TinySkiaBackend {}

impl GraphicsBackend for TinySkiaBackend {
    fn bbox(&mut self, pos: Cursor, width: f64, height: f64, role: Role) {
        if let Some(rect) = Rect::from_xywh(pos.x as f32, pos.y as f32, width as f32, height as f32) {
            let color = match role {
                Role::Glyph => Color::from_rgba8(0, 200, 0, 255),
                Role::HBox => Color::from_rgba8(200, 0, 0,  255),
                Role::VBox => Color::from_rgba8(0, 0, 200,  255),
            };
            self.paint.set_color(color);

            let path = {
                let mut path_builder = PathBuilder::new();
                path_builder.push_rect(rect);
                path_builder.finish().unwrap()
            };

            let mut stroke = Stroke::default();
            stroke.width = 0.; // hairline stroking to avoid scaling issues
            self.pixmap.stroke_path(
                &path, 
                &self.paint, 
                &stroke, 
                self.layout_to_pixmap, 
                None
            );
            self.paint.set_color(self.current_color);
        }
    }
    fn rule(&mut self, pos: Cursor, width: f64, height: f64) {
        if let Some(rect) = Rect::from_xywh(pos.x as f32, pos.y as f32, width as f32, height as f32) {
            self.pixmap.fill_rect(
                rect,
                &self.paint,
                self.layout_to_pixmap,
                None,
            );
        }
    }
    fn begin_color(&mut self, RGBA(r, g, b, a): RGBA) {
        self.color_stack.push(self.current_color);
        self.current_color = Color::from_rgba8(r, g, b, a);
        self.paint.set_color(self.current_color);
    }
    fn end_color(&mut self) {
        if let Some(next_color) = self.color_stack.pop() {
            self.current_color = next_color;
            self.paint.set_color(self.current_color);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::font::backend::ttf_parser::TtfMathFont;
    use crate::layout::Style;
    use crate::LayoutBuilder;
    use crate::Renderer;

    #[cfg(feature="ttfparser-fontparser")]
    fn load_font<'a>(file: &'a [u8]) -> crate::font::backend::ttf_parser::TtfMathFont<'a> {
        let font = ttf_parser::Face::parse(file, 0).unwrap();
        TtfMathFont::new(font).unwrap()
    }
    #[test]
    #[cfg(feature="ttfparser-fontparser")]
    fn test_tiny_skia_backend_ttfparser() {
        // TODO: Add tests for TinySkiaBackend
        let font_file: &[u8] = include_bytes!("../../resources/XITS_Math.otf");
        let font = load_font(font_file);
        let equation = "x_f = \\sqrt{\\frac{a + b}{c - d}}";
        const FONT_SIZE: f64 = 16.0;
        let layout_engine = LayoutBuilder::new(&font)
            .font_size(FONT_SIZE)
            .style(Style::Display)
            .build();

        let parse_nodes = crate::parser::parse(equation).unwrap();

        let layout = layout_engine.layout(&parse_nodes).unwrap();

        let renderer = Renderer::new();

        const SCALE: f64 = 5.;
        let mut tinyskia_backend = TinySkiaBackend::from_dims(layout.size(), SCALE).unwrap();
        renderer.render(&layout, &mut tinyskia_backend);
        let save_location = std::env::temp_dir().join("ttfparser-tinyskia.png");
        tinyskia_backend
            .pixmap()
            .save_png(&save_location)
            .unwrap();
        println!("Saved PNG file to {:?}", save_location);
    }

    #[test]
    #[cfg(feature="fontrs-fontparser")]
    fn test_tiny_skia_backend_fontrs() {
        let font_file: &[u8] = include_bytes!("../../resources/FiraMath_Regular.otf");
        let font = OpenTypeFont::parse(font_file).unwrap();
        let equation = "x_f = {\\color{red}\\sqrt{\\frac{a + b}{c - d}}}";
        const FONT_SIZE: f64 = 16.0;
        let layout_engine = LayoutBuilder::new(&font)
            .font_size(FONT_SIZE)
            .style(Style::Display)
            .build();

        let parse_nodes = crate::parser::parse(equation).unwrap();

        let layout = layout_engine.layout(&parse_nodes).unwrap();

        let mut renderer = Renderer::new();
        renderer.debug = true;

        const SCALE: f64 = 5.;
        let mut tinyskia_backend = TinySkiaBackend::from_dims(layout.size(), SCALE).unwrap();
        tinyskia_backend.set_color(Color::WHITE);
        renderer.render(&layout, &mut tinyskia_backend);
        let save_location = std::env::temp_dir().join("fontrs-tinyskia.png");
        tinyskia_backend
            .pixmap()
            .save_png(&save_location)
            .unwrap();
        println!("Saved PNG file to {:?}", save_location);
    }
}
