use font::opentype::OpenTypeFont;

use crate::font::{Constants, Glyph};
use crate::font::common::VariantGlyph;

use crate::{dimensions::*, font::common::GlyphId};
use crate::error::FontError;

use crate::font::kerning::Corner;
use crate::font::IsMathFont;


pub type MathFont = OpenTypeFont;



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
