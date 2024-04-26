/// Obtain kerning measurements for subscripts and superscripts.
/// The kerning is the amount of space that one must add between the symbol and its super-/sub-script.
#[deny(missing_docs)]
pub mod kerning;

/// Different implementations of the 'MathFont' trait for various font parsing crates, like 'ttf-parser'.
#[deny(missing_docs)]
pub mod backend;
/// Contains types and utilities related to fonts.
/// In particular, defines utilities related to extended glyphs, i.e. glyphs like '}' and '→', which can be made bigger.
#[deny(missing_docs)]
pub mod common;
mod style;
//mod unit;

pub use unicode_math::AtomType;
pub use style::style_symbol;


pub use crate::font::common::{Direction, VariantGlyph};

use crate::{font::common::GlyphId};
use crate::dimensions::Unit;
use crate::dimensions::units::{Em, FUnit, Ratio};
use crate::error::FontError;

use self::kerning::Corner;


pub trait MathFont : Sized {
    fn glyph_index(&self, codepoint: char) -> Option<crate::font::common::GlyphId>;
    fn glyph_from_gid<'f>(&'f self, glyph_id : GlyphId) -> Result<Glyph<'f, Self>, FontError>;
    fn kern_for(&self, glyph_id : GlyphId, height : Unit<FUnit>, side : Corner) -> Option<Unit<FUnit>>;

    fn italics(&self, glyph_id : GlyphId) -> i16;
    fn attachment(&self, glyph_id : GlyphId) -> i16; 
    fn constants(&self, font_units_to_em: Unit<Ratio<Em, FUnit>>) -> Constants;
    fn font_units_to_em(&self) -> Unit<Ratio<Em, FUnit>>;


    fn horz_variant(&self, gid: GlyphId, width: Unit<FUnit>)  -> VariantGlyph;
    // TODO : there seems to be a problem in "qc.rs" 
    // the } before "wat?" is too short for the last 2 fonts but not the first
    // maybe this is a problem, maybe this is meant to be
    fn vert_variant(&self, gid: GlyphId, height: Unit<FUnit>) -> VariantGlyph;
}

pub struct FontContext<'f, F> {
    pub font: &'f F,
    pub constants: Constants,
    pub units_per_em: Unit<Ratio<FUnit, Em>>,
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

impl<'f, F : MathFont> FontContext<'f, F> {
    pub fn new(font: &'f F) -> Self {
        let font_units_to_em = font.font_units_to_em();
        let units_per_em = font_units_to_em.recip();
        let constants = font.constants(font_units_to_em);

        FontContext {
            font,
            units_per_em,
            constants
        }
    }

    pub fn glyph(&self, codepoint: char) -> Result<Glyph<'f, F>, FontError> {
        let gid = self.font.glyph_index(codepoint).ok_or(FontError::MissingGlyphCodepoint(codepoint))?;
        self.glyph_from_gid(gid)
    }



    pub fn vert_variant(&self, codepoint: char, height: Unit<FUnit>) -> Result<VariantGlyph, FontError> {
        let gid = self.font.glyph_index(codepoint).ok_or(FontError::MissingGlyphCodepoint(codepoint))?;
        Ok(self.font.vert_variant(gid, height))
    }
    pub fn horz_variant(&self, codepoint: char, width: Unit<FUnit>) -> Result<VariantGlyph, FontError> {
        let gid = self.font.glyph_index(codepoint).ok_or(FontError::MissingGlyphCodepoint(codepoint))?;
        Ok(self.font.horz_variant(gid, width))
    }

    pub fn glyph_from_gid(&self, gid: GlyphId) -> Result<Glyph<'f, F>, FontError> {
        self.font.glyph_from_gid(gid)
    }
}


#[derive(Clone)]
pub struct Constants {
    pub subscript_shift_down: Unit<Em>,
    pub subscript_top_max: Unit<Em>,
    pub subscript_baseline_drop_min: Unit<Em>,

    pub superscript_baseline_drop_max: Unit<Em>,
    pub superscript_bottom_min: Unit<Em>,
    pub superscript_shift_up_cramped: Unit<Em>,
    pub superscript_shift_up: Unit<Em>,
    pub sub_superscript_gap_min: Unit<Em>,

    pub upper_limit_baseline_rise_min: Unit<Em>,
    pub upper_limit_gap_min: Unit<Em>,
    pub lower_limit_gap_min: Unit<Em>,
    pub lower_limit_baseline_drop_min: Unit<Em>,

    pub fraction_rule_thickness: Unit<Em>,
    pub fraction_numerator_display_style_shift_up: Unit<Em>,
    pub fraction_denominator_display_style_shift_down: Unit<Em>,
    pub fraction_num_display_style_gap_min: Unit<Em>,
    pub fraction_denom_display_style_gap_min: Unit<Em>,
    pub fraction_numerator_shift_up: Unit<Em>,
    pub fraction_denominator_shift_down: Unit<Em>,
    pub fraction_numerator_gap_min: Unit<Em>,
    pub fraction_denominator_gap_min: Unit<Em>,

    pub axis_height: Unit<Em>,
    pub accent_base_height: Unit<Em>,

    pub delimited_sub_formula_min_height: Unit<Em>,
    pub display_operator_min_height: Unit<Em>,

    pub radical_display_style_vertical_gap: Unit<Em>,
    pub radical_vertical_gap: Unit<Em>,
    pub radical_rule_thickness: Unit<Em>,
    pub radical_extra_ascender: Unit<Em>,

    pub stack_display_style_gap_min: Unit<Em>,
    pub stack_top_display_style_shift_up: Unit<Em>,
    pub stack_top_shift_up: Unit<Em>,
    pub stack_bottom_shift_down: Unit<Em>,
    pub stack_gap_min: Unit<Em>,

    pub delimiter_factor: f64,
    pub delimiter_short_fall: Unit<Em>,
    pub null_delimiter_space: Unit<Em>,

    pub script_percent_scale_down: f64,
    pub script_script_percent_scale_down: f64,
}


pub struct Glyph<'f, F> {
    pub font: &'f F,
    pub gid:  GlyphId,
    // x_min, y_min, x_max, y_max
    pub bbox: (Unit<FUnit>, Unit<FUnit>, Unit<FUnit>, Unit<FUnit>),
    pub advance: Unit<FUnit>,
    pub lsb: Unit<FUnit>,
    pub italics: Unit<FUnit>,
    pub attachment: Unit<FUnit>,
}
impl<'f, F> Glyph<'f, F> {
    pub fn height(&self) -> Unit<FUnit> {
        self.bbox.3
    }
    pub fn depth(&self) -> Unit<FUnit> {
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
