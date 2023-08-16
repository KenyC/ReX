use std::convert::TryFrom;

use font::opentype::OpenTypeFont;

use crate::font::{Constants, Glyph};
use crate::font::common::VariantGlyph;

use crate::{font::common::GlyphId};
use crate::dimensions::Unit;
use crate::dimensions::units::{Em, FUnit, Ratio};
use crate::error::FontError;

use crate::font::kerning::Corner;
use crate::font::MathFont;





impl MathFont for OpenTypeFont {

    fn glyph_index(&self, codepoint: char) -> Option<GlyphId> {
        use font::Font;
        let result = self.gid_for_codepoint(codepoint as u32)?;
        Some(GlyphId::try_from(result).ok()?)
    }

    fn glyph_from_gid<'f>(&'f self, gid: GlyphId) -> Result<Glyph<'f, Self>, FontError> {
        use font::Font as FontTrait;
        let font = self;
        let hmetrics = font.glyph_metrics(gid.into()).ok_or(FontError::MissingGlyphGID(gid))?;
        let italics = self.italics(gid);
        let attachment = self.attachment(gid);
        let glyph = font.glyph(gid.into()).ok_or(FontError::MissingGlyphGID(gid))?;
        let bbox = glyph.path.bounds();
        let ll = bbox.lower_left();
        let ur = bbox.upper_right();

        Ok(Glyph {
            gid,
            font: self,
            advance:    Unit::<FUnit>::new(hmetrics.advance.into()),
            lsb:        Unit::<FUnit>::new(hmetrics.lsb.into()),
            italics:    Unit::<FUnit>::new(italics.into()),
            attachment: Unit::<FUnit>::new(attachment.into()),
            bbox: (
                Unit::<FUnit>::new(ll.x().into()),
                Unit::<FUnit>::new(ur.y().into()),
                Unit::<FUnit>::new(ur.x().into()),
                Unit::<FUnit>::new(ll.y().into()),
            )
        })
    }

    fn kern_for(&self, glyph_id : GlyphId, height : Unit<FUnit>, side : Corner) -> Option<Unit<FUnit>> {
        let math = self.math.as_ref().unwrap();
        let record = math.glyph_info.kern_info.entries.get(&glyph_id.into())?;

        let table = match side {
            Corner::TopRight => &record.top_right,
            Corner::TopLeft => &record.top_left,
            Corner::BottomRight => &record.bottom_right,
            Corner::BottomLeft => &record.bottom_left,
        };

        Some(Unit::<FUnit>::new(table.kern_for_height((height.unitless(FUnit)) as i16).into()))
    }




    fn italics(&self, glyph_id : GlyphId) -> i16 {
        self.math
            .as_ref()
            .unwrap()
            .glyph_info
            .italics_correction_info
            .get(glyph_id.into())
            .map(|info| info.value)
            .unwrap_or_default()
    }

    fn attachment(&self, gid: GlyphId) -> i16 {
        self
            .math
            .as_ref()
            .unwrap()
            .glyph_info
            .top_accent_attachment
            .get(gid.into())
            .map(|info| info.value)
            .unwrap_or_default()
    }

    fn constants(&self, font_units_to_em: Unit<Ratio<Em, FUnit>>) -> Constants {
        let em = |v: f64| -> Unit<Em> { Unit::<FUnit>::new(v) * font_units_to_em };

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
            delimiter_short_fall: Unit::<Em>::new(0.1),
            null_delimiter_space: Unit::<Em>::new(0.1),

            script_percent_scale_down: 0.01 * f64::from(math_constants.script_percent_scale_down),
            script_script_percent_scale_down: 0.01 * f64::from(math_constants.script_script_percent_scale_down),
        }
    }

    fn font_units_to_em(&self) -> Unit<Ratio<Em, FUnit>> {
        use font::Font as FontTrait;
        Unit::<Ratio<Em, FUnit>>::new(self.font_matrix().matrix.m11() as f64)
    }

    fn horz_variant(&self, gid: GlyphId, width: Unit<FUnit>) -> VariantGlyph {
        self
            .math
            .as_ref()
            .unwrap()
            .variants
            .horz_variant(gid.into(), (width.unitless(FUnit)) as u32)
            .into()
    }

    fn vert_variant(&self, gid: GlyphId, height: Unit<FUnit>) -> VariantGlyph {
        self
            .math
            .as_ref()
            .unwrap()
            .variants
            .vert_variant(gid.into(), (height.unitless(FUnit)) as u32)
            .into()
    }

}
