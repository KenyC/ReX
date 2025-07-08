//! Defines a renderer that does not draw anything but computes the "real bbox", the one the encloses all areas actually drawn to.
//! Character (typically in italic style) routinely go beyond the bounding box as defined in the font. 
//! To determine the real bounding box, you can perform a renderer with the backend defined in this module.


use crate::{dimensions::{units::{Em, FUnit, Px, Ratio}, Unit}, font::{backend::ttf_parser::TtfMathFont, common::GlyphId, MathFont}, geometry::BBox};

use super::{Backend, FontBackend, GraphicsBackend};



/// A rendering backend that does not draw but simply records strokes
#[derive(Debug, Clone)]
pub struct BBoxBackend {
    bbox: BBox<Px>,
}

impl Default for BBoxBackend {
    fn default() -> Self {
        Self::new(0., 0.)
    }
}

impl BBoxBackend {
    /// Creates a new bbox rendering backend.
    pub fn new(x_origin : f64, y_origin : f64) -> Self {
        Self { 
            bbox: BBox { 
                x_min: Unit::<Px>::new(x_origin), 
                x_max: Unit::<Px>::new(x_origin), 
                y_min: Unit::<Px>::new(y_origin), 
                y_max: Unit::<Px>::new(y_origin), 
            }
        }
    }
}

impl GraphicsBackend for BBoxBackend {
    fn rule(&mut self, pos: super::Cursor, width: f64, height: f64) {
        self.bbox = self.bbox.union(BBox::from_dims(
            Unit::new(pos.x), Unit::new(pos.y),
            Unit::new(width), Unit::new(height),
        ));
    }

    fn begin_color(&mut self, _color: super::RGBA) {} 
    fn end_color(&mut self) {}
}

impl<F : MathFont> FontBackend<F> for BBoxBackend {
    fn symbol(&mut self, pos: crate::Cursor, gid: GlyphId, scale: f64, ctx: &F) {

        let scale = Unit::<Ratio<Px, Em>>::new(scale); 
        let em_per_funits = ctx.font_units_to_em();
        let (x_min, y_min, x_max, y_max) = ctx.glyph_from_gid(gid).unwrap().bbox;

        let font_bbox = BBox::<FUnit>::new(x_min, -y_max, x_max, -y_min);

        let x = Unit::<Px>::new(pos.x);
        let y = Unit::<Px>::new(pos.y);

        let funits_to_px: Unit<Ratio<Px, FUnit>> = em_per_funits * scale.lift();
        let bbox : BBox<Px> = 
            font_bbox
            .scale(funits_to_px)
            .translate(x, y)
        ;

        self.bbox = self.bbox.union(bbox);
    }
}

impl<F : MathFont> Backend<F> for BBoxBackend {}

#[cfg(test)]
mod tests {
    use crate::font::backend::ttf_parser::TtfMathFont;

    use super::*;


    /// Percentage of error compared to values
    // const ERROR_TOLERANCE : f64 = 10.;
    const ERROR_TOLERANCE : f64 = 1e-3;

    #[cfg(feature="ttfparser-fontparser")]
    #[test]
    fn test_symbol_bbox() {
        use crate::{font::MathFont, Cursor};

        let font_file = include_bytes!("../../resources/Garamond_Math.otf");
        let font = ttf_parser::Face::parse(font_file, 0).expect("Couldn't parse font.");
        let math_font = TtfMathFont::new(font).expect("The font likely lacks a MATH table"); // extracts math info from font


        let mut bbox_backend = BBoxBackend::default();

        let gid = math_font.glyph_index('\u{1D453}').unwrap(); // 𝑓
        let font_size = Unit::<Ratio<Px, Em>>::new(1.);

        bbox_backend.symbol(
            Cursor { x: 0., y: 0. }, 
            gid, 
            font_size.to_unitless(), // 1 pixel per em
            &math_font
        );

        // Values gathered from FontForge
        // funits per em = 1000
        // Then I found the extremal contour points of the outline (by visual inspection!) and noted their coordinates in funits
        let expected = BBox::<FUnit> { 
            x_min: Unit::new(-97.), 
            x_max: Unit::new(600.), 
            y_min: Unit::new(-290.), 
            y_max: Unit::new(706.), 
        }.scale(Unit::<Ratio<FUnit, Em>>::new(1000.).recip() * font_size.lift());
        assert!(
            BBox::close_to(
                &bbox_backend.bbox,
                &expected,
                ERROR_TOLERANCE,
            ),
            "expected: {:?}, found: {:?}",
            expected,
            bbox_backend.bbox,
        )   


    }

    #[cfg(feature="ttfparser-fontparser")]
    #[test]
    fn test_complex_formula_bbox() {
        use crate::{layout::engine::DEFAULT_FONT_SIZE, render};

        let font_file = include_bytes!("../../resources/Garamond_Math.otf");
        let font = ttf_parser::Face::parse(font_file, 0).expect("Couldn't parse font.");
        let math_font = TtfMathFont::new(font).expect("The font likely lacks a MATH table"); // extracts math info from font






        /* 
        Values are obtained by first rendering the svg using (12 is DEFAULT_FONT_SIZE):
        
        ```
        cargo r --example svg-basic --all-features -- "f+f"  -f resources/Garamond_Math.otf -s 12
        ```
        
        Then process the svg with inkscape:

        ```
        inkscape test.svg -X -Y -W -H
        ```
        
        and recodring the values in the console.

        */
        assert_eq!(DEFAULT_FONT_SIZE, Unit::new(12.));

        // TODO: fix this
        // Because `svg-basic` places origin on top of the baseline, we translate our results
        // This means that we are not testing position of baseline with this test
        let mut bbox_backend = BBoxBackend::default();
        render(
            "f+f", 
            &mut bbox_backend, 
            &math_font
        ).unwrap();
        let expected = BBox::<Px>::from_dims( 
            Unit::new(-1.55078),
            Unit::new(0.),
            Unit::new(37.0938),
            Unit::new(15.9375),
        // In addition, Inkscape doesn't include the origin so we must include in the bbox
        ).union(BBox::single_point(Unit::ZERO, Unit::ZERO));
        let found = bbox_backend.bbox.translate(Unit::ZERO, -bbox_backend.bbox.y_min);
        assert!(
            BBox::close_to(
                &found,
                &expected,
                ERROR_TOLERANCE,
            ),
            "expected: {:#?}, found: {:#?}",
            expected,
            found,
        );

        let mut bbox_backend = BBoxBackend::default();
        render(
            r"\int_{-\infty}^{\infty} \frac{\sin(x)}{x}\,\mathrm{d}x = \int_{-\infty}^{\infty}\frac{\sin^2(x)}{x^2}\,\mathrm{d}x", 
            &mut bbox_backend, 
            &math_font
        ).unwrap();
        let expected = BBox::<Px>::from_dims( 
            Unit::new(1.28125),
            Unit::new(0.),
            Unit::new(201.997),
            Unit::new(37.1953),
        ).union(BBox::single_point(Unit::ZERO, Unit::ZERO));
        let found = bbox_backend.bbox.translate(Unit::ZERO, -bbox_backend.bbox.y_min);
        assert!(
            BBox::close_to(
                &found,
                &expected,
                ERROR_TOLERANCE,
            ),
            "expected: {:#?}, found: {:#?}",
            expected,
            found,
        )   


    }
}