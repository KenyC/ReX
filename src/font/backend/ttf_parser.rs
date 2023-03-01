use ttf_parser::{math::{GlyphPart, Variants}, LazyArray16};

use crate::{font::{Constants, VariantGlyph, common::{GlyphInstruction, GlyphId}, Direction, Glyph}, dimensions::{Scale, Font, Em, Length}, error::FontError};


pub struct TtfMathFont<'a> {
    math: ttf_parser::math::Table<'a>,
    font: ttf_parser::Face<'a>,
    font_matrix: ttf_parser::cff::Matrix,
}

impl<'a> TtfMathFont<'a> {
    pub fn new(font: ttf_parser::Face<'a>) -> Result<Self, FontError> { 
        let math = font.tables().math.ok_or(FontError::NoMATHTable)?;
        let font_matrix; 
        if let Some(cff) = font.tables().cff {
            font_matrix = cff.matrix();
        }
        else {
            let units_per_em = font.tables().head.units_per_em;
            font_matrix = ttf_parser::cff::Matrix {
                sx: (units_per_em as f32).recip(),
                ky: 0.,
                kx: 0.,
                sy: (units_per_em as f32).recip(),
                tx: 0.,
                ty: 0.,
            };
        };
        Ok(Self { 
            math, 
            font,
            font_matrix,
        }) 
    }
    
    pub fn font(&self) -> &ttf_parser::Face<'a> {
        &self.font
    }

    pub fn font_matrix(&self) -> ttf_parser::cff::Matrix {
        self.font_matrix
    }
}


impl<'a> TtfMathFont<'a> {
    fn safe_italics(&self, glyph_id : GlyphId) -> Option<i16> {
        let value = self.math.glyph_info?
            .italic_corrections?
            .get(glyph_id.into())?
            .value;
        Some(value)
    }

    fn safe_attachment(&self, glyph_id : GlyphId) -> Option<i16> {
        // TODO : cache GlyphInfo table & constants
        let value = self.math.glyph_info?
            .top_accent_attachments?
            .get(glyph_id.into())?
            .value;
        Some(value)
    }

    fn safe_constants(&self, font_units_to_em : Scale<Em, Font>) -> Option<Constants> {
        // perhaps cache : GlyphInfo table
        let math_constants = self.math.constants?;
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



impl<'a> crate::font::MathFont for TtfMathFont<'a> {
    fn italics(&self, glyph_id : GlyphId) -> i16 {
        self.safe_italics(glyph_id).unwrap_or_default()
    }

    fn attachment(&self, glyph_id : GlyphId) -> i16 {
        self.safe_attachment(glyph_id).unwrap_or_default()
    }

    fn constants(&self, font_units_to_em: Scale<Em, Font>) -> Constants {
        self.safe_constants(font_units_to_em).unwrap()
    }

    fn horz_variant(&self, gid: GlyphId, height: crate::dimensions::Length<Font>) -> crate::font::common::VariantGlyph {
        // NOTE: The following is an adaptation of the corresponding code in the crate "font"
        // NOTE: bizarrely, the code for horizontal variant is not isomorphic to the code for vertical variant ; here, I've simply adapted the vertical variant code
        // TODO: figure out why horiz_variant uses 'greatest_lower_bound' and vert_variant uses 'smallest_lowerè_bound'
        let variants = match self.math.variants {
            Some(variants) => variants,
            None => return VariantGlyph::Replacement(gid),
        };

        // If the font does not specify a construction of vertical variant of a glyph, the glyph will be used as is
        let construction = match variants.horizontal_constructions.get(gid.into()) {
            Some(construction) => construction,
            None => return VariantGlyph::Replacement(gid),
        };


        // Otherwise, check if any replacement glyphs are larger than the demanded size: we use them if they exist.
        for record in construction.variants {
            if record.advance_measurement >= (height / Font) as u16 {
                return VariantGlyph::Replacement(GlyphId::from(record.variant_glyph));
            }
        }

        // Otherwise, check if there is a generic recipe for building large glyphs
        // If not take, the largest replacement glyph if there is one
        // we are in the generic case ; the glyph must be constructed from glyph parts
        let glyph_id = construction.variants.last().map(|v| GlyphId::from(v.variant_glyph)).unwrap_or(gid);
        let replacement = VariantGlyph::Replacement(glyph_id);
        let assembly = match construction.assembly {
            None => {
                return replacement;
            },
            Some(ref assembly) => assembly,
        };

        let size = (height / Font) as u32;
        if let Some((repeats, diff_ratio)) = greatest_lower_bound(&variants, assembly.parts, size) {
            let instructions = construct_glyphs(&variants, assembly.parts, repeats, diff_ratio);
            VariantGlyph::Constructable(Direction::Horizontal, instructions)
        }
        else {
            trace!("constructable glyphs are too large");
            replacement
        }

    }


    fn vert_variant(&self, gid: GlyphId, height: crate::dimensions::Length<Font>) -> crate::font::common::VariantGlyph {
        // NOTE: The following is an adaptation of the corresponding code in the crate "font"

        let variants = match self.math.variants {
            Some(variants) => variants,
            None => return VariantGlyph::Replacement(gid),
        };

        // If the font does not specify a construction of vertical variant of a glyph, the glyph will be used as is
        let construction = match variants.vertical_constructions.get(gid.into()) {
            Some(construction) => construction,
            None => return VariantGlyph::Replacement(gid),
        };


        // Otherwise, check if any replacement glyphs are larger than the demanded size: we use them if they exist.
        for record in construction.variants {
            if record.advance_measurement >= (height / Font) as u16 {
                return VariantGlyph::Replacement(GlyphId::from(record.variant_glyph));
            }
        }

        // Otherwise, check if there is a generic recipe for building large glyphs
        // If not take, the largest replacement glyph if there is one
        // we are in the generic case ; the glyph must be constructed from glyph parts
        let map = construction.variants.last().map(|v| GlyphId::from(v.variant_glyph)).unwrap_or(gid);
        let replacement = VariantGlyph::Replacement(map);
        let assembly = match construction.assembly {
            None => {
                return replacement;
            },
            Some(ref assembly) => assembly,
        };

        let size = (height / Font) as u32;
        let (repeats, diff_ratio) = smallest_upper_bound(&variants, assembly.parts, size);
        let instructions = construct_glyphs(&variants, assembly.parts, repeats, diff_ratio);

        VariantGlyph::Constructable(Direction::Vertical, instructions)
    }

    fn glyph_index(&self, codepoint: char) -> Option<crate::font::common::GlyphId> {
        let glyph_index_ttf_parser = self.font.glyph_index(codepoint)?;
        Some(crate::font::common::GlyphId::from(glyph_index_ttf_parser))
    }

    fn glyph_from_gid<'f>(&'f self, gid : GlyphId) -> Result<crate::font::Glyph<'f, Self>, FontError> {
        let glyph_id : ttf_parser::GlyphId = gid.into();
        let bbox     = self.font.glyph_bounding_box(glyph_id).ok_or(FontError::MissingGlyphGID(gid))?;
        let advance  = self.font.glyph_hor_advance(glyph_id).ok_or(FontError::MissingGlyphGID(gid))?;
        let lsb  = self.font.glyph_hor_side_bearing(glyph_id).ok_or(FontError::MissingGlyphGID(gid))?;
        let italics = self.italics(gid);
        let attachment = self.attachment(gid);
        Ok(Glyph {
            font: self,
            gid,
            bbox: (
                Length::new(bbox.x_min, Font), 
                Length::new(bbox.y_min, Font), 
                Length::new(bbox.x_max, Font), 
                Length::new(bbox.y_max, Font),
            ),
            advance:    Length::new(advance, Font),
            lsb:        Length::new(lsb,     Font),
            italics:    Length::new(italics, Font),
            attachment: Length::new(attachment, Font),

        })
    }

    fn kern_for(&self, glyph_id : GlyphId, height : Length<Font>, side : crate::font::kerning::Corner) -> Option<Length<Font>> {
        let record = self.math.glyph_info?.kern_infos?.get(glyph_id.into())?;

        let table = match side {
            crate::font::kerning::Corner::TopRight    => record.top_right.as_ref(),
            crate::font::kerning::Corner::TopLeft     => record.top_left.as_ref(),
            crate::font::kerning::Corner::BottomRight => record.bottom_right.as_ref(),
            crate::font::kerning::Corner::BottomLeft  => record.bottom_left.as_ref(),
        }?;


        let mut i = 0;
        while let Some((h, kern)) = Option::zip(table.height(i), table.kern(i)) {
            if height < Length::new(h.value, Font) {
                return Some(Length::new(kern.value, Font));
            }
            i += 1;
        }

        Some(Length::new(table.kern(i - 1).unwrap().value, Font))
    }

    fn font_units_to_em(&self) -> Scale<Em, Font> {
        Scale::new(self.font_matrix.sx as f64, Em, Font)
    }


}



fn max_overlap(variants : &Variants, left: u16, right: &GlyphPart) -> u16 {
    // NOTE: The following is an adaptation of the corresponding code in the crate "font"
    let overlap = std::cmp::min(left, right.start_connector_length);
    let overlap = std::cmp::min(overlap, right.full_advance / 2);
    std::cmp::max(overlap, variants.min_connector_overlap)
}

fn construct_glyphs(variants : &Variants, parts: LazyArray16<GlyphPart>, repeats: u16, diff_ratio: f64) -> Vec<GlyphInstruction> {
    // NOTE: The following is an adaptation of the corresponding code in the crate "font"

    // Construct the variant glyph
    let mut prev_connector = 0;
    let mut first = true;
    trace!("diff: {:?}, repeats: {}", diff_ratio, repeats);

    let mut to_return = Vec::with_capacity(repeats as usize + 3);
    for glyph in parts {
        let repeat = if glyph.part_flags.extender() { repeats } else { 1 };
        for _ in 0..repeat {
            let overlap = if first {
                first = false;
                0
            } else {
                // linear interpolation
                //  d * max_overlap + (1 - d) * MIN_CONNECTOR_OVERLAP
                let max = max_overlap(variants, prev_connector, &glyph);
                let overlap = (1.0 - diff_ratio) * max as f64
                    + diff_ratio * variants.min_connector_overlap as f64;
                overlap as u16
            };
            prev_connector = std::cmp::min(glyph.end_connector_length, glyph.full_advance / 2);

            to_return.push(GlyphInstruction {
                gid: GlyphId::from(glyph.glyph_id),
                overlap: overlap
            });
        }
    }

    to_return
}

/// Construct the smallest variant that is larger than the given size.
/// With the number of glyphs required to construct the variant is larger
/// than `ITERATION_LIMIT` we return `None`.
fn smallest_upper_bound(variants : &Variants, parts: LazyArray16<GlyphPart>, size: u32) -> (u16, f64) {
    // NOTE: The following is an adaptation of the corresponding code in the crate "font"

    let (small, large) = advance_without_optional(variants, parts);
    if small < size {
        trace!("using smallest variant glyph, {} <= smallest <= {}", small, large);
        return (0, 0.0)
    }

    // Otherwise, check the next largest variant with optional glyphs included.
    let (mut small, mut large, opt_small, opt_large) = advance_with_optional(variants,parts);
    if large >= size {
        let diff_ratio = f64::from(size - small) / f64::from(large - small);
        trace!("Optional glyphs: 1, Difference ratio: {:2}", diff_ratio);
        return (1, diff_ratio);
    } 

    // We need to find the smallest integer k that satisfies:
    //     large + k * opt_large >= size
    // This is solved by:
    //     (size - large) / opt_large <= k
    // So take k = ceil[ (size - large) / opt_large ]
    let k = u32::from( (size - large) / opt_large ) + 1;
    trace!("k = ({} - {}) / {} = {}", size, large, opt_large, k);
    small += k * opt_small;
    large += k * opt_large;
    trace!("new size: {} <= advance <= {}", small, large);

    //  A---o---B, percentage: (o - A) / (B - A)
    // o  A-----B, percentage: 0 (smallest glyph).
    // Need small + diff_ratio * (opt_large - opt_small) = size
    if small >= size {
        return (k as u16 + 1, 0.into());
    }

    let difference_ratio = f64::from(size - small) / f64::from(large - small);
    trace!("Difference ratio: ({:?} - {:?}) / ({:?} - {:?}) = {:?}",
        size, small, large, small, difference_ratio);
    trace!("New size: {} + {} * {} * {}", small, k, difference_ratio, opt_large - opt_small);
    (k as u16 + 1, difference_ratio)
}

/// Calculate the advance of the smallest variant with exactly one set of optional
/// connectors. This returns a tuple: the first element states the advance of a
/// variant with one set of optional connectors, the second element states the
/// increase in advance for each additional connector.
fn advance_with_optional(variants : &Variants, parts: LazyArray16<GlyphPart>) -> (u32, u32, u32, u32) {
    // NOTE: The following is an adaptation of the corresponding code in the crate "font"

    let mut advance_small = 0;
    let mut advance_large = variants.min_connector_overlap as u32;
    let mut connector_small = 0;
    let mut connector_large = 0;
    let mut prev_connector = 0;

    // Calculate the legnth with exactly one connector
    for glyph in parts {
        let overlap = max_overlap(variants, prev_connector, &glyph);
        advance_small += (glyph.full_advance - overlap) as u32;
        advance_large += (glyph.full_advance - variants.min_connector_overlap) as u32;
        prev_connector = std::cmp::min(glyph.end_connector_length, glyph.full_advance / 2);

        // Keep record of the advance each additional connector adds
        if glyph.part_flags.extender() {
            let overlap = max_overlap(variants, glyph.start_connector_length, &glyph);
            connector_small += (glyph.full_advance - overlap) as u32;
            connector_large += (glyph.full_advance - variants.min_connector_overlap) as u32;
        }
    }

    trace!("variant with optional glyphs: {} <= advance <= {}", advance_small, advance_large);
    trace!("advance from optional glyphs: {} <= advance <= {}",
        connector_small, connector_large);
    (advance_small, advance_large, connector_small, connector_large)
}

fn advance_without_optional(variants : &Variants, parts: LazyArray16<GlyphPart>) -> (u32, u32) {
    // NOTE: The following is an adaptation of the corresponding code in the crate "font"

    let mut advance_small = 0;
    let mut advance_large = variants.min_connector_overlap as u32;
    let mut prev_connector = 0;

    for glyph in parts.into_iter().filter(|glyph| glyph.part_flags.extender()) {
        let overlap = max_overlap(variants, prev_connector, &glyph);
        advance_small += (glyph.full_advance - overlap) as u32;
        advance_large += (glyph.full_advance - variants.min_connector_overlap) as u32;
        prev_connector = std::cmp::min(glyph.end_connector_length, glyph.full_advance / 2);
    }

    (advance_small, advance_large)
}


/// Measure the _largest_ a glyph construction _smaller_ than the given size. 
/// If all constructions are larger than the given size, return `None`.
/// Otherwise return the number of optional glyphs required and the difference
/// ratio to obtain the desired size.
fn greatest_lower_bound(
    variants : &Variants,
    parts:     LazyArray16<GlyphPart>, 
    size:      u32
) 
-> Option<(u16, f64)> 
{
    let (small, large) = advance_without_optional(variants, parts);
    if small >= size {
        trace!("all constructable glyphs are too large, smallest: {}", small);
        return None;
    }

    // Otherwise calculate the size of including one set of optional glyphs.
    let (mut ssmall, mut llarge, opt_small, opt_large) = advance_with_optional(variants, parts);

    // If the smallest constructable with optional glyphs is too large we
    // use no optional glyphs.
    // TODO: Do something better if `large == small`.
    if ssmall >= size {
        let diff_ratio = f64::from(size - small) / f64::from(large - small);
        let diff_ratio = diff_ratio.min(1.0);
        trace!("optional glyphs make construction too large, using none");
        trace!("diff_ratio = {:.2}", diff_ratio);
        return Some((0, diff_ratio));
    }

    // Determine the number of additional optional glyphs required to achieve size.
    // We need to find the smallest integer k such that:
    //     ssmall + k*opt_small >= size
    // This is solved by:
    //     (size - ssmall) / opt_small <= k
    // Which is solved by: k = floor[ (size - smmal) / opt_small ]
    // Since we round towards zero, floor is not necessary.
    let k = (size - ssmall) / opt_small;
    trace!("k = ({} - {})/ {} = {}", size, ssmall, opt_small, k);

    ssmall += k * opt_small;
    llarge += k * opt_large;
    let diff_ratio = f64::from(size - ssmall) / f64::from(llarge - ssmall);
    let diff_ratio = diff_ratio.min(1.0).max(0.0);

    trace!("{} <= advance <= {}", ssmall, llarge);
    trace!("Difference ratio: {}", diff_ratio);
    Some((k as u16 + 1, diff_ratio))
}

