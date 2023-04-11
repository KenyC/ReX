use cairo::Context;

use crate::{Backend, font::backend::ttf_parser::TtfMathFont, GraphicsBackend, FontBackend, parser::color};

pub struct CairoBackend {
    context : Context,
    current_color : (u8, u8, u8, u8),
    color_stack   : Vec<(u8, u8, u8, u8)>,
}

impl CairoBackend {
    pub fn new(context: Context) -> Self {
        context.set_source_rgba(0., 0., 0., 1.);
        Self {
            context,
            current_color: (0x00, 0x00, 0x00, 0xff),
            color_stack: Vec::new(),
        }
    }

    pub fn context(self) -> Context
    {self.context}

    fn set_current_color(&mut self) {
        let (r, g, b, a,) = self.current_color;
        #[inline]
        fn u8_to_f64(x : u8) -> f64 { f64::from(x) / 255. }
        self.context.set_source_rgba(u8_to_f64(r), u8_to_f64(g), u8_to_f64(b), u8_to_f64(a),);
    }
}


#[cfg(feature="ttfparser-fontparser")]
impl<'a> Backend<TtfMathFont<'a>> for CairoBackend {}


impl GraphicsBackend for CairoBackend {
    fn bbox(&mut self, _pos: crate::Cursor, _width: f64, _height: f64, role: crate::Role) {
        match role {
            crate::Role::Glyph => self.context.set_source_rgb(0., 0.785, 0.),
            crate::Role::VBox  => self.context.set_source_rgb(0.785, 0., 0.),
            crate::Role::HBox  => self.context.set_source_rgb(0., 0., 0.785),
        }
        self.context.set_line_width(1.0);
        self.context.rectangle(_pos.x, _pos.y, _width, _height);
        self.context.stroke().unwrap();
        self.set_current_color();
    }

    fn rule(&mut self, pos: crate::Cursor, width: f64, height: f64) {
        let context = &self.context;
        context.rectangle(pos.x, pos.y, width, height);
        context.fill().unwrap();
    }


    fn begin_color(&mut self, color: crate::RGBA) {
        self.current_color = (color.0, color.1, color.2, color.3,);
        self.set_current_color();
    }

    fn end_color(&mut self) {
        if let Some(color) = self.color_stack.pop() {
            self.current_color = color;
            self.set_current_color();
        }
    }
}


#[cfg(feature="ttfparser-fontparser")]
impl<'a> FontBackend<TtfMathFont<'a>> for CairoBackend {
    fn symbol(&mut self, pos: crate::Cursor, gid: crate::font::common::GlyphId, scale: f64, ctx: &TtfMathFont<'a>) {
        use ttf_parser::OutlineBuilder;


        let context = &self.context;
        context.save().unwrap();
        context.translate(pos.x, pos.y);
        context.scale(scale, -scale);
        context.scale(ctx.font_matrix().sx.into(), ctx.font_matrix().sy.into(),);
        context.set_fill_rule(cairo::FillRule::EvenOdd);
        context.new_path();

        struct Builder<'a> { 
            // path   : Path,
            // paint  : Paint,
            context : &'a Context,
        }

        impl<'a> Builder<'a> {
            fn fill(self) {
                self.context.fill().unwrap();
            }
        }

        impl<'a> OutlineBuilder for Builder<'a> {
            fn move_to(&mut self, x: f32, y: f32) {
                // eprintln!("move_to {:?} {:?}", x, y);
                self.context.move_to(x.into(), y.into());
            }

            fn line_to(&mut self, x: f32, y: f32) {
                // eprintln!("line_to {:?} {:?}", x, y);
                self.context.line_to(x.into(), y.into());
            }

            fn quad_to(&mut self, x1: f32, y1: f32, x: f32, y: f32) {
                // eprintln!("quad_to  {:?} {:?} {:?} {:?}", x1, y1, x, y);
                self.context.curve_to(x1.into(), y1.into(), x1.into(), y1.into(), x.into(), y.into(),)
            }

            fn curve_to(&mut self, x1: f32, y1: f32, x2: f32, y2: f32, x: f32, y: f32) {
                // eprintln!("curve_to {:?} {:?} {:?} {:?} {:?} {:?}", x1, y1, x2, y2, x, y);
                self.context.curve_to(x1.into(), y1.into(), x2.into(), y2.into(), x.into(), y.into(),)
            }

            fn close(&mut self) {
                // eprintln!("close");
                self.context.close_path();
            }

        }

        let mut builder = Builder {
            context: context,
        };

        ctx.font().outline_glyph(gid.into(), &mut builder);
        builder.fill();
        context.restore().unwrap();
    }
}