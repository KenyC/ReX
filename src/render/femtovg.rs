use femtovg::{Renderer, Canvas, Solidity, Path, Paint};
#[cfg(feature="fontcrate-backend")]
use font::{Font, OpenTypeFont};
#[cfg(feature="fontcrate-backend")]
use pathfinder_content::{outline::ContourIterFlags, segment::SegmentKind};
#[cfg(feature="fontcrate-backend")]
use pathfinder_geometry::{transform2d::Transform2F, vector::Vector2F};

use crate::{Backend, font::common::GlyphId};



#[cfg(feature="fontcrate-backend")]
fn v_cursor(c: crate::Cursor) -> Vector2F {
    Vector2F::new(c.x as f32, c.y as f32)
}
#[cfg(feature="fontcrate-backend")]
fn v_xy(x: f64, y: f64) -> Vector2F {
    Vector2F::new(x as f32, y as f32)
}

pub struct FemtoVGCanvas<'a, T : Renderer> {
    canvas : &'a mut Canvas<T>,
    current_paint : femtovg::Paint,
    color_stack: Vec<femtovg::Paint>,
}

impl<'a, T: Renderer> FemtoVGCanvas<'a, T> {
    pub fn new(
        canvas: &'a mut Canvas<T>, 
        current_paint: femtovg::Paint,
    ) -> Self { 
        Self { 
            canvas, 
            current_paint, 
            color_stack : Vec::new() 
        } 
    }

    pub fn canvas<'b>(&'b mut self) -> &'b mut Canvas<T> {
        self.canvas
    }
}

#[cfg(feature="fontcrate-backend")]
impl<'a, T : Renderer> Backend<OpenTypeFont> for FemtoVGCanvas<'a, T> {
    fn symbol(&mut self, pos: crate::Cursor, gid: GlyphId, scale: f64, ctx: &OpenTypeFont) {
        let path = ctx.glyph(gid.into()).unwrap().path;
        let tr = Transform2F::from_translation(v_cursor(pos))
            * Transform2F::from_scale(v_xy(scale, -scale))
            * ctx.font_matrix();
        // println!("{:?} {:?}", gid,  tr);
        println!("{:?} {:?} {:?} {:?}", gid,  pos.x, pos.y, scale);
        let path = path.transformed(&tr);


        let contours = path.into_contours();
        let mut contour_path = femtovg::Path::new();
        for contour in contours {
            // println!("### CONTOUR ######################");

            if let Some(segment) = contour.iter(ContourIterFlags::empty()).next() {
                let baseline = segment.baseline;
                contour_path.move_to(baseline.from_x(), baseline.from_y());
            }

            for segment in contour.iter(ContourIterFlags::empty()) {
                // println!("{:?}", segment);
                let baseline = segment.baseline;
                let control  = segment.ctrl;
                match segment.kind {
                    SegmentKind::None => (),
                    SegmentKind::Line => {
                        // contour_path.move_to(baseline.from_x(), baseline.from_y());
                        contour_path.line_to(baseline.to_x(),   baseline.to_y());
                    },
                    SegmentKind::Quadratic => {
                        // contour_path.move_to(baseline.from_x(), baseline.from_y());
                        contour_path.quad_to(
                            control.from_x(),  control.from_y(), 
                            baseline.to_x(), baseline.to_y()
                        );
                    },
                    SegmentKind::Cubic => {
                        // contour_path.move_to(baseline.from_x(), baseline.from_y());
                        contour_path.bezier_to(
                            control.from_x(),  control.from_y(), 
                            control.to_x(),    control.to_y(), 
                            baseline.to_x(),   baseline.to_y()
                        );
                    },
                }
            }
            if contour.is_closed() {
                contour_path.close();
            }
        }
        self.canvas.fill_path(&mut contour_path, &self.current_paint);
        // todo!()
    }

    fn rule(&mut self, pos: crate::Cursor, width: f64, height: f64) {
        let mut path = femtovg::Path::new();
        path.rect(pos.x as f32, pos.y as f32, width as f32, height as f32);

        self.canvas.fill_path(&mut path, &self.current_paint)
    }

    fn begin_color(&mut self, color: crate::RGBA) {
        let color = femtovg::Color::rgba(color.0, color.1, color.2, color.3);
        let paint = femtovg::Paint::color(color)
            .with_anti_alias(true)
            .with_fill_rule(femtovg::FillRule::EvenOdd)
        ;
        let old_paint = std::mem::replace(&mut self.current_paint, paint);
        self.color_stack.push(old_paint);
    }

    fn end_color(&mut self) {
        self.current_paint = self.color_stack.pop().unwrap();
    }


}

#[cfg(feature = "ttfparser-backend")]
impl<'a, 'f, T : Renderer> Backend<crate::font::backend::ttf_parser::MathFont<'f>> for FemtoVGCanvas<'a, T> {

    fn symbol(&mut self, pos: crate::Cursor, gid: GlyphId, scale: f64, ctx: &crate::font::backend::ttf_parser::MathFont<'f>) {
        use ttf_parser::OutlineBuilder;

        let scale = scale as f32;
        self.canvas.save();
        self.canvas.translate(pos.x as f32, pos.y as f32);
        self.canvas.scale(scale, - scale);
        self.canvas.scale(ctx.font_matrix().sx, ctx.font_matrix().sy,);
        // self.canvas.scale(0.01, - 0.01);

        struct Builder<'a, T : Renderer> { 
            path   : Path,
            paint  : Paint,
            canvas : &'a mut Canvas<T>
        }

        impl<'a, T: Renderer> Builder<'a, T> {
            fn fill(self) {
                let Self { mut path, paint, canvas } = self;
                canvas.fill_path(&mut path, &paint);
            }
        }

        impl<'a, T : Renderer> OutlineBuilder for Builder<'a, T> {
            fn move_to(&mut self, x: f32, y: f32) {
                // println!("move_to {:?} {:?}", x, y);
                self.path.move_to(x, y);
            }

            fn line_to(&mut self, x: f32, y: f32) {
                // println!("line_to {:?} {:?}", x, y);
                self.path.line_to(x, y);
            }

            fn quad_to(&mut self, x1: f32, y1: f32, x: f32, y: f32) {
                // println!("quad_to  {:?} {:?} {:?} {:?}", x1, y1, x, y);
                self.path.quad_to(x1, y1, x, y);
            }

            fn curve_to(&mut self, x1: f32, y1: f32, x2: f32, y2: f32, x: f32, y: f32) {
                // println!("curve_to {:?} {:?} {:?} {:?} {:?} {:?}", x1, y1, x2, y2, x, y);
                self.path.curve_to(x1, y1, x2, y2, x, y);
            }

            fn close(&mut self) {
                // println!("close");
                self.path.close();
            }

        }

        let Self { canvas, current_paint, .. } = self;
        let mut builder = Builder {
            path:   Path::new(),
            paint:  current_paint.clone(),
            canvas: canvas,
        };

        ctx.font().outline_glyph(gid.into(), &mut builder);
        builder.fill();
        self.canvas.restore();
    }

    fn rule(&mut self, pos: crate::Cursor, width: f64, height: f64) {
        let mut path = femtovg::Path::new();
        path.rect(pos.x as f32, pos.y as f32, width as f32, height as f32);

        self.canvas.fill_path(&mut path, &self.current_paint)
    }

    fn begin_color(&mut self, color: crate::RGBA) {
        let color = femtovg::Color::rgba(color.0, color.1, color.2, color.3);
        let paint = femtovg::Paint::color(color)
            .with_anti_alias(true)
            .with_fill_rule(femtovg::FillRule::EvenOdd)
        ;
        let old_paint = std::mem::replace(&mut self.current_paint, paint);
        self.color_stack.push(old_paint);
    }

    fn end_color(&mut self) {
        self.current_paint = self.color_stack.pop().unwrap();
    }
}