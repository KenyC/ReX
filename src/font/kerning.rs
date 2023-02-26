use super::{Glyph};
use std::cmp::{max, min};

use crate::{dimensions::{Length, Font}, MathFont};

#[derive(Debug)]
enum Corner {
    TopRight,
    TopLeft,
    BottomRight,
    BottomLeft,
}

// Horizontal Position:
//     - By default, set flat to base glyph
//     - For superscript, add italics correction from base character
//     - For suprscript:
//         - Calculate bottom of script (after shiftup)
//         - Calculate top of base.
//     - For subscript:
//         - Calculate top of script (after shift down)
//         - Calculate bottom of base
//     - For each script:
//         - Find math kern value at this height for base.
//           (TopRight for superscript, BotRight for subscript)
//         - Find math kern value at this height for sciprt.
//           (BotLeft for subscript, TopRight for superscript)
//         - Add the values together together
//     - Horintal kern is applied to smallest of two results
//       from previous step.

// I question the accuracy of this algorithm.  But it's not yet clear to me what
// the relavent values should represent with respect to the "cut-ins" for the kerning.
// for now, I'm just going to port the algorithm I found in LuaTeX and XeTeX.
// If nothing else, it will at least be consistent.

pub fn superscript_kern(base: &Glyph<MathFont>, script: &Glyph<MathFont>, shift: Length<Font>) -> Length<Font> {
    let base_height = base.bbox.3;
    let script_depth = script.bbox.1 + shift;

    let value1 = kern_from(base, base_height, Corner::TopRight) +
    kern_from(script, base_height, Corner::BottomLeft);

    let value2 = kern_from(base, script_depth, Corner::TopRight) +
    kern_from(script, script_depth, Corner::BottomLeft);

    max(value1, value2)
}

pub fn subscript_kern(base: &Glyph<MathFont>, script: &Glyph<MathFont>, shift: Length<Font>) -> Length<Font> {
    let base_depth = base.bbox.1;
    let script_height = script.bbox.3 - shift;

    let value1 = kern_from(base, base_depth, Corner::BottomRight) +
    kern_from(script, base_depth, Corner::TopLeft);

    let value2 = kern_from(base, script_height, Corner::BottomRight) +
    kern_from(script, script_height, Corner::TopLeft);

    min(value1, value2)
}

fn kern_from(glyph: &Glyph<MathFont>, height: Length<Font>, side: Corner) -> Length<Font> {
    let math = glyph.font.math.as_ref().unwrap();
    let record = match math.glyph_info.kern_info.entries.get(&glyph.gid) {
        Some(record) => record,
        None => return Length::zero(),
    };

    let table = match side {
        Corner::TopRight => &record.top_right,
        Corner::TopLeft => &record.top_left,
        Corner::BottomRight => &record.bottom_right,
        Corner::BottomLeft => &record.bottom_left,
    };

    Length::new(table.kern_for_height((height / Font) as i16), Font)
}