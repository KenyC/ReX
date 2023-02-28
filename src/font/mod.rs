pub mod kerning;
pub mod backend;
pub mod common;
mod style;
//mod unit;

pub use unicode_math::AtomType;
pub use style::style_symbol;


use font::opentype::OpenTypeFont;
pub use font::opentype::math::MathConstants;
pub use crate::font::common::{Direction, VariantGlyph};

use crate::{dimensions::*, font::common::GlyphId};
use crate::error::FontError;

use self::kerning::Corner;

pub type MathFont = OpenTypeFont;

// TODO: when "font" dependency is expunged, rename as "MathFont"
pub trait IsMathFont : Sized {
    fn glyph_index(&self, codepoint: char) -> Option<crate::font::common::GlyphId>;
    fn glyph_from_gid<'f>(&'f self, glyph_id : u16) -> Result<Glyph<'f, Self>, FontError>;
    fn kern_for(&self, glyph_id : u16, height : Length<Font>, side : Corner) -> Option<Length<Font>>;

    fn italics(&self, glyph_id : u16) -> i16;
    fn attachment(&self, glyph_id : u16) -> i16; 
    fn constants(&self, font_units_to_em: Scale<Em, Font>) -> Constants;
    fn font_units_to_em(&self) -> Scale<Em, Font>;


    fn horz_variant(&self, gid: u32, width: Length<Font>)  -> VariantGlyph;
    // TODO : there seems to be a problem in "qc.rs" 
    // the } before "wat?" is too short for the last 2 fonts but not the first
    // maybe this is a problem, maybe this is meant to be
    fn vert_variant(&self, gid: u32, height: Length<Font>) -> VariantGlyph;
}

impl IsMathFont for MathFont {

    fn glyph_index(&self, codepoint: char) -> Option<crate::font::common::GlyphId> {
        use font::Font;
        let result = self.gid_for_codepoint(codepoint as u32)?;
        Some(result.into())
    }

    fn glyph_from_gid<'f>(&'f self, gid: u16) -> Result<Glyph<'f, Self>, FontError> {
        use font::Font;
        let font = self;
        let hmetrics = font.glyph_metrics(gid).ok_or(FontError::MissingGlyphGID(gid))?;
        let italics = self.italics(gid);
        let attachment = self.attachment(gid);
        let glyph = font.glyph(GlyphId(gid as u32).into()).ok_or(FontError::MissingGlyphGID(gid))?;
        let bbox = glyph.path.bounds();
        let ll = bbox.lower_left();
        let ur = bbox.upper_right();

        Ok(Glyph {
            gid,
            font: self,
            advance: Length::new(hmetrics.advance, Font),
            lsb: Length::new(hmetrics.lsb, Font),
            italics: Length::new(italics, Font),
            attachment: Length::new(attachment, Font),
            bbox: (
                Length::new(ll.x(), Font),
                Length::new(ur.y(), Font),
                Length::new(ur.x(), Font),
                Length::new(ll.y(), Font),
            )
        })
    }

    fn kern_for(&self, glyph_id : u16, height : Length<Font>, side : Corner) -> Option<Length<Font>> {
        let math = self.math.as_ref().unwrap();
        let record = math.glyph_info.kern_info.entries.get(&glyph_id)?;

        let table = match side {
            Corner::TopRight => &record.top_right,
            Corner::TopLeft => &record.top_left,
            Corner::BottomRight => &record.bottom_right,
            Corner::BottomLeft => &record.bottom_left,
        };

        Some(Length::new(table.kern_for_height((height / Font) as i16), Font))
    }




    fn italics(&self, glyph_id : u16) -> i16 {
        self.math
            .as_ref()
            .unwrap()
            .glyph_info
            .italics_correction_info
            .get(glyph_id)
            .map(|info| info.value)
            .unwrap_or_default()
    }

    fn attachment(&self, gid: u16) -> i16 {
        self
            .math
            .as_ref()
            .unwrap()
            .glyph_info
            .top_accent_attachment
            .get(gid)
            .map(|info| info.value)
            .unwrap_or_default()
    }

    fn constants(&self, font_units_to_em: Scale<Em, Font>) -> Constants {
        let em = |v: f64| -> Length<Em> { Length::new(v, Font) * font_units_to_em };

        let math_constants = &self
            .math
            .as_ref()
            .unwrap()
            .constants
        ;
        Constants {
            subscript_shift_down: em(math_constants.subscript_top_max.value.into()),
            subscript_top_max: em(math_constants.subscript_top_max.value.into()),
            subscript_baseline_drop_min: em(math_constants.subscript_baseline_drop_min.value.into()),
            
            superscript_baseline_drop_max: em(math_constants.superscript_baseline_drop_max.value.into()),
            superscript_bottom_min: em(math_constants.superscript_bottom_min.value.into()),
            superscript_shift_up_cramped: em(math_constants.superscript_shift_up_cramped.value.into()),
            superscript_shift_up: em(math_constants.superscript_shift_up.value.into()),
            sub_superscript_gap_min: em(math_constants.sub_superscript_gap_min.value.into()),

            upper_limit_baseline_rise_min: em(math_constants.upper_limit_baseline_rise_min.value.into()),
            upper_limit_gap_min: em(math_constants.upper_limit_gap_min.value.into()),
            lower_limit_gap_min: em(math_constants.lower_limit_gap_min.value.into()),
            lower_limit_baseline_drop_min: em(math_constants.lower_limit_baseline_drop_min.value.into()),

            fraction_rule_thickness: em(math_constants.fraction_rule_thickness.value.into()),
            fraction_numerator_display_style_shift_up: em(math_constants.fraction_numerator_display_style_shift_up.value.into()),
            fraction_denominator_display_style_shift_down: em(math_constants.fraction_denominator_display_style_shift_down.value.into()),
            fraction_num_display_style_gap_min: em(math_constants.fraction_num_display_style_gap_min.value.into()),
            fraction_denom_display_style_gap_min: em(math_constants.fraction_denom_display_style_gap_min.value.into()),
            fraction_numerator_shift_up: em(math_constants.fraction_numerator_shift_up.value.into()),
            fraction_denominator_shift_down: em(math_constants.fraction_denominator_shift_down.value.into()),
            fraction_numerator_gap_min: em(math_constants.fraction_numerator_gap_min.value.into()),
            fraction_denominator_gap_min: em(math_constants.fraction_denominator_gap_min.value.into()),

            axis_height: em(math_constants.axis_height.value.into()),
            accent_base_height: em(math_constants.accent_base_height.value.into()),

            delimited_sub_formula_min_height: em(math_constants.delimited_sub_formula_min_height.into()),

            display_operator_min_height: em(math_constants.display_operator_min_height.into()),

            radical_display_style_vertical_gap: em(math_constants.radical_display_style_vertical_gap.value.into()),
            radical_vertical_gap: em(math_constants.radical_vertical_gap.value.into()),
            radical_rule_thickness: em(math_constants.radical_rule_thickness.value.into()),
            radical_extra_ascender: em(math_constants.radical_extra_ascender.value.into()),

            stack_display_style_gap_min: em(math_constants.stack_display_style_gap_min.value.into()),
            stack_top_display_style_shift_up: em(math_constants.stack_top_display_style_shift_up.value.into()),
            stack_top_shift_up: em(math_constants.stack_top_shift_up.value.into()),
            stack_bottom_shift_down: em(math_constants.stack_bottom_shift_down.value.into()),
            stack_gap_min: em(math_constants.stack_gap_min.value.into()),

            // TODO: trait implementations should not be allowed to vary on these values
            delimiter_factor: 0.901,
            delimiter_short_fall: Length::new(0.1, Em),
            null_delimiter_space: Length::new(0.1, Em),

            script_percent_scale_down: 0.01 * f64::from(math_constants.script_percent_scale_down),
            script_script_percent_scale_down: 0.01 * f64::from(math_constants.script_script_percent_scale_down),
        }
    }

    fn font_units_to_em(&self) -> Scale<Em, Font> {
        use font::Font;
        Scale::new(self.font_matrix().matrix.m11() as f64, Em, Font)
    }

    fn horz_variant(&self, gid: u32, width: Length<Font>) -> VariantGlyph {
        self
            .math
            .as_ref()
            .unwrap()
            .variants
            .horz_variant(gid as u16, (width / Font) as u32)
            .into()
    }

    fn vert_variant(&self, gid: u32, height: Length<Font>) -> VariantGlyph {
        self
            .math
            .as_ref()
            .unwrap()
            .variants
            .vert_variant(gid as u16, (height / Font) as u32)
            .into()
    }

}


pub struct FontContext<'f, F> {
    pub font: &'f F,
    pub constants: Constants,
    pub units_per_em: Scale<Font, Em>,
}

impl<'f, F> Clone for FontContext<'f, F> {
    fn clone(&self) -> Self {
        Self {
            font:         self.font,
            constants:    self.constants.clone(),
            units_per_em: self.units_per_em,
        }
    }
}

impl<'f, F : IsMathFont> FontContext<'f, F> {
    pub fn new(font: &'f F) -> Result<Self, FontError> {
        use font::Font;
        let font_units_to_em = font.font_units_to_em();
        let units_per_em = font_units_to_em.inv();
        let constants = font.constants(font_units_to_em);

        Ok(FontContext {
            font,
            units_per_em,
            constants
        })
    }

    pub fn glyph(&self, codepoint: char) -> Result<Glyph<'f, F>, FontError> {
        let gid = self.font.glyph_index(codepoint).ok_or(FontError::MissingGlyphCodepoint(codepoint))?;
        self.glyph_from_gid(gid.0 as u16)
    }



    pub fn vert_variant(&self, codepoint: char, height: Length<Font>) -> Result<VariantGlyph, FontError> {
        let GlyphId(gid) = self.font.glyph_index(codepoint).ok_or(FontError::MissingGlyphCodepoint(codepoint))?;
        Ok(self.font.vert_variant(gid, height))
    }
    pub fn horz_variant(&self, codepoint: char, width: Length<Font>) -> Result<VariantGlyph, FontError> {
        let GlyphId(gid) = self.font.glyph_index(codepoint).ok_or(FontError::MissingGlyphCodepoint(codepoint))?;
        Ok(self.font.horz_variant(gid, width))
    }

    pub fn glyph_from_gid(&self, gid: u16) -> Result<Glyph<'f, F>, FontError> {
        self.font.glyph_from_gid(gid)
    }
}


#[derive(Clone)]
pub struct Constants {
    pub subscript_shift_down: Length<Em>,
    pub subscript_top_max: Length<Em>,
    pub subscript_baseline_drop_min: Length<Em>,

    pub superscript_baseline_drop_max: Length<Em>,
    pub superscript_bottom_min: Length<Em>,
    pub superscript_shift_up_cramped: Length<Em>,
    pub superscript_shift_up: Length<Em>,
    pub sub_superscript_gap_min: Length<Em>,

    pub upper_limit_baseline_rise_min: Length<Em>,
    pub upper_limit_gap_min: Length<Em>,
    pub lower_limit_gap_min: Length<Em>,
    pub lower_limit_baseline_drop_min: Length<Em>,

    pub fraction_rule_thickness: Length<Em>,
    pub fraction_numerator_display_style_shift_up: Length<Em>,
    pub fraction_denominator_display_style_shift_down: Length<Em>,
    pub fraction_num_display_style_gap_min: Length<Em>,
    pub fraction_denom_display_style_gap_min: Length<Em>,
    pub fraction_numerator_shift_up: Length<Em>,
    pub fraction_denominator_shift_down: Length<Em>,
    pub fraction_numerator_gap_min: Length<Em>,
    pub fraction_denominator_gap_min: Length<Em>,

    pub axis_height: Length<Em>,
    pub accent_base_height: Length<Em>,

    pub delimited_sub_formula_min_height: Length<Em>,
    pub display_operator_min_height: Length<Em>,

    pub radical_display_style_vertical_gap: Length<Em>,
    pub radical_vertical_gap: Length<Em>,
    pub radical_rule_thickness: Length<Em>,
    pub radical_extra_ascender: Length<Em>,

    pub stack_display_style_gap_min: Length<Em>,
    pub stack_top_display_style_shift_up: Length<Em>,
    pub stack_top_shift_up: Length<Em>,
    pub stack_bottom_shift_down: Length<Em>,
    pub stack_gap_min: Length<Em>,

    pub delimiter_factor: f64,
    pub delimiter_short_fall: Length<Em>,
    pub null_delimiter_space: Length<Em>,

    pub script_percent_scale_down: f64,
    pub script_script_percent_scale_down: f64,
}


pub struct Glyph<'f, F> {
    pub font: &'f F,
    pub gid: u16,
    // x_min, y_min, x_max, y_max
    pub bbox: (Length<Font>, Length<Font>, Length<Font>, Length<Font>),
    pub advance: Length<Font>,
    pub lsb: Length<Font>,
    pub italics: Length<Font>,
    pub attachment: Length<Font>,
}
impl<'f, F> Glyph<'f, F> {
    pub fn height(&self) -> Length<Font> {
        self.bbox.3
    }
    pub fn depth(&self) -> Length<Font> {
        self.bbox.1
    }
}

#[derive(Default, Debug, Copy, Clone, PartialEq, Eq)]
pub struct Style {
    pub family: Family,
    pub weight: Weight,
}


impl Style {
    pub fn new() -> Style {
        Style::default()
    }

    pub fn with_family(self, fam: Family) -> Style {
        Style {
            family: fam,
            ..self
        }
    }

    pub fn with_weight(self, weight: Weight) -> Style {
        Style {
            weight: weight,
            ..self
        }
    }

    pub fn with_bold(self) -> Style {
        Style {
            weight: self.weight.with_bold(),
            ..self
        }
    }

    pub fn with_italics(self) -> Style {
        Style {
            weight: self.weight.with_italics(),
            ..self
        }
    }
}

// NB: Changing the order of these variants requires
//     changing the LUP in fontselection
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum Family {
    Roman,
    Script,
    Fraktur,
    SansSerif,
    Blackboard,
    Monospace,
    Normal,
}

// NB: Changing the order of these variants requires
//     changing the LUP in fontselection
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum Weight {
    None,
    Italic,
    Bold,
    BoldItalic,
}

impl Weight {
    fn with_bold(self) -> Self {
        match self {
            Weight::Italic | Weight::BoldItalic => Weight::BoldItalic,
            _ => Weight::Bold,
        }
    }

    fn with_italics(self) -> Self {
        match self {
            Weight::Bold | Weight::BoldItalic => Weight::BoldItalic,
            _ => Weight::Italic,
        }
    }
}

impl Default for Family {
    fn default() -> Family {
        Family::Normal
    }
}

impl Default for Weight {
    fn default() -> Weight {
        Weight::None
    }
}
