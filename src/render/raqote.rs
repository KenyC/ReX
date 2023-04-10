use raqote::{Color, DrawTarget, Source, SolidSource, DrawOptions, Transform, PathBuilder};

use crate::{Backend, font::backend::ttf_parser::TtfMathFont, GraphicsBackend, FontBackend};

pub struct RaqoteBackend<'a> {
    target        : &'a mut DrawTarget,
    current_color : SolidSource,
    color_stack   : Vec<SolidSource>,
}

impl<'a> RaqoteBackend<'a> {
    pub fn new(target: &'a mut DrawTarget) -> Self {
        Self {
            target,
            current_color: SolidSource::from_unpremultiplied_argb(0xff, 0x00, 0x00, 0x00),
            color_stack:   Vec::new(),
        }
    }

    fn push_color(&mut self, r : u8, g : u8, b: u8, a : u8) {
        self.color_stack.push(self.current_color);
        self.current_color = SolidSource {r, g, b, a,};
    }

    fn pop_color(&mut self) {
        if let Some(color) = self.color_stack.pop() {
            self.current_color = color;
        }
    }
}


#[cfg(feature="ttfparser-fontparser")]
impl<'a, 'dt> Backend<TtfMathFont<'a>> for RaqoteBackend<'dt> {}


impl<'dt> GraphicsBackend for RaqoteBackend<'dt> {
    fn bbox(&mut self, pos: crate::Cursor, width: f64, height: f64, role: crate::Role) {
        match role {
            crate::Role::Glyph => self.begin_color(crate::RGBA(0x00, 0xc1, 0x00, 0xff,)),
            crate::Role::VBox  => self.begin_color(crate::RGBA(0xc1, 0x00, 0x00, 0xff,)),
            crate::Role::HBox  => self.begin_color(crate::RGBA(0x00, 0x00, 0xc1, 0xff,)),
        }
        let mut path_builder = raqote::PathBuilder::new();
        path_builder.rect(pos.x as f32, pos.y as f32, width as f32, height as f32);
        let path = path_builder.finish();

        self.target.fill(&path, &Source::Solid(self.current_color), &DrawOptions::default());
        self.end_color();
    }

    fn rule(&mut self, pos: crate::Cursor, width: f64, height: f64) {
        let mut path_builder = raqote::PathBuilder::new();
        path_builder.rect(pos.x as f32, pos.y as f32, width as f32, height as f32);
        let path = path_builder.finish();

        self.target.fill(&path, &Source::Solid(self.current_color), &DrawOptions::default());
    }


    fn begin_color(&mut self, color: crate::RGBA) {
        self.color_stack.push(self.current_color);
        self.current_color = SolidSource {
            r : color.0, 
            g : color.1, 
            b : color.2, 
            a : color.3,
        };
    }

    fn end_color(&mut self) {
        if let Some(color) = self.color_stack.pop() {
            self.current_color = color;
        }
    }
}


#[cfg(feature="ttfparser-fontparser")]
impl<'a, 'dt> FontBackend<TtfMathFont<'a>> for RaqoteBackend<'dt> {
    fn symbol(&mut self, pos: crate::Cursor, gid: crate::font::common::GlyphId, scale: f64, ctx: &TtfMathFont<'a>) {
        use ttf_parser::OutlineBuilder;

        let font_matrix = ctx.font_matrix();
        let transform =
            Transform::translation(pos.x as f32, pos.y as f32)
            .pre_scale(scale as f32, - scale as f32)
            // .then_scale(scale as f32, - scale as f32)
            .pre_scale(font_matrix.sx, font_matrix.sy)
        ;

        struct Builder { 
            path_builder : PathBuilder,
        }

        impl Builder {
            fn new() -> Self { 
                Self { 
                    path_builder : PathBuilder::new(),
                } 
            }
        }


        impl<'a> OutlineBuilder for Builder {
            fn move_to(&mut self, x: f32, y: f32) {
                self.path_builder.move_to(x, y);
            }

            fn line_to(&mut self, x: f32, y: f32) {
                self.path_builder.line_to(x, y);
            }

            fn quad_to(&mut self, x1: f32, y1: f32, x: f32, y: f32) {
                self.path_builder.quad_to(x1, y1, x, y)
            }

            fn curve_to(&mut self, x1: f32, y1: f32, x2: f32, y2: f32, x: f32, y: f32) {
                self.path_builder.cubic_to(x1, y1, x2, y2, x, y)
            }

            fn close(&mut self) {
                self.path_builder.close();
            }

        }

        let mut builder = Builder::new();
        ctx.font().outline_glyph(gid.into(), &mut builder);

        let path = builder.path_builder.finish().transform(&transform);
        

        self.target.fill(
            &path, 
            &Source::Solid(self.current_color), 
            &DrawOptions {
                blend_mode: raqote::BlendMode::SrcOver,
                alpha: 1.,
                antialias: raqote::AntialiasMode::Gray,
            },
        );

    }
}