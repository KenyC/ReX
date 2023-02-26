


#[cfg(feature="ttfparser-backend")]
pub mod ttf_parser {

    use ttf_parser::GlyphId;

    use crate::{font::Constants, dimensions::{Scale, Font, Em, Length}};


    pub struct MathFont {
    }

    pub struct MathHeader<'a>(ttf_parser::math::Table<'a>);

    impl<'a> MathHeader<'a> {
        fn safe_italics(&self, glyph_id : u16) -> Option<i16> {
            let value = self.0.glyph_info?
                .italic_corrections?
                .get(GlyphId(glyph_id))?
                .value;
            Some(value)
        }

        fn safe_attachment(&self, glyph_id : u16) -> Option<i16> {
            // TODO : cache GlyphInfo table & constants
            let value = self.0.glyph_info?
                .top_accent_attachments?
                .get(GlyphId(glyph_id))?
                .value;
            Some(value)
        }

        fn safe_constants(&self, font_units_to_em : Scale<Em, Font>) -> Option<Constants> {
            // perhaps cache : GlyphInfo table
            let math_constants = self.0.constants?;
            let em = |v: f64| -> Length<Em> { Length::new(v, Font) * font_units_to_em };


            Some(Constants {
                subscript_shift_down:        em(math_constants.subscript_top_max().value.into()),
                subscript_top_max:           em(math_constants.subscript_top_max().value.into()),
                subscript_baseline_drop_min: em(math_constants.subscript_baseline_drop_min().value.into()),
                
                superscript_baseline_drop_max: em(math_constants.superscript_baseline_drop_max().value.into()),
                superscript_bottom_min:        em(math_constants.superscript_bottom_min().value.into()),
                superscript_shift_up_cramped:  em(math_constants.superscript_shift_up_cramped().value.into()),
                superscript_shift_up:          em(math_constants.superscript_shift_up().value.into()),
                sub_superscript_gap_min:       em(math_constants.sub_superscript_gap_min().value.into()),

                upper_limit_baseline_rise_min: em(math_constants.upper_limit_baseline_rise_min().value.into()),
                upper_limit_gap_min:           em(math_constants.upper_limit_gap_min().value.into()),
                lower_limit_gap_min:           em(math_constants.lower_limit_gap_min().value.into()),
                lower_limit_baseline_drop_min: em(math_constants.lower_limit_baseline_drop_min().value.into()),

                fraction_rule_thickness:                       em(math_constants.fraction_rule_thickness().value.into()),
                fraction_numerator_display_style_shift_up:     em(math_constants.fraction_numerator_display_style_shift_up().value.into()),
                fraction_denominator_display_style_shift_down: em(math_constants.fraction_denominator_display_style_shift_down().value.into()),
                fraction_num_display_style_gap_min:            em(math_constants.fraction_num_display_style_gap_min().value.into()),
                fraction_denom_display_style_gap_min:          em(math_constants.fraction_denom_display_style_gap_min().value.into()),
                fraction_numerator_shift_up:                   em(math_constants.fraction_numerator_shift_up().value.into()),
                fraction_denominator_shift_down:               em(math_constants.fraction_denominator_shift_down().value.into()),
                fraction_numerator_gap_min:                    em(math_constants.fraction_numerator_gap_min().value.into()),
                fraction_denominator_gap_min:                  em(math_constants.fraction_denominator_gap_min().value.into()),

                axis_height:        em(math_constants.axis_height().value.into()),
                accent_base_height: em(math_constants.accent_base_height().value.into()),

                delimited_sub_formula_min_height: em(math_constants.delimited_sub_formula_min_height().into()),

                display_operator_min_height: em(math_constants.display_operator_min_height().into()),

                radical_display_style_vertical_gap: em(math_constants.radical_display_style_vertical_gap().value.into()),
                radical_vertical_gap:               em(math_constants.radical_vertical_gap().value.into()),
                radical_rule_thickness:             em(math_constants.radical_rule_thickness().value.into()),
                radical_extra_ascender:             em(math_constants.radical_extra_ascender().value.into()),

                stack_display_style_gap_min:      em(math_constants.stack_display_style_gap_min().value.into()),
                stack_top_display_style_shift_up: em(math_constants.stack_top_display_style_shift_up().value.into()),
                stack_top_shift_up:               em(math_constants.stack_top_shift_up().value.into()),
                stack_bottom_shift_down:          em(math_constants.stack_bottom_shift_down().value.into()),
                stack_gap_min:                    em(math_constants.stack_gap_min().value.into()),

                delimiter_factor: 0.901,
                delimiter_short_fall: Length::new(0.1, Em),
                null_delimiter_space: Length::new(0.1, Em),


                script_percent_scale_down: 0.01 * f64::from(math_constants.script_percent_scale_down()),
                script_script_percent_scale_down: 0.01 * f64::from(math_constants.script_script_percent_scale_down()),

            })
        }
    }



    impl<'a> crate::font::IsMathHeader for MathHeader<'a> {
        fn italics(&self, glyph_id : u16) -> i16 {
            self.safe_italics(glyph_id).unwrap_or_default()
        }

        fn attachment(&self, glyph_id : u16) -> i16 {
            self.safe_attachment(glyph_id).unwrap_or_default()
        }

        fn constants(&self, font_units_to_em: Scale<Em, Font>) -> Constants {
            self.safe_constants(font_units_to_em).unwrap()
        }

        fn horz_variant(&self, gid: u32, width: crate::dimensions::Length<Font>) -> crate::font::common::VariantGlyph {
            todo!()
        }

        fn vert_variant(&self, gid: u32, height: crate::dimensions::Length<Font>) -> crate::font::common::VariantGlyph {
            todo!()
        }

    }

}