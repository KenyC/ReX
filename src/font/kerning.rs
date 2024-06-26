use super::{Glyph, MathFont};


use crate::dimensions::{units::{FUnit}, Unit};

/// Corners of a glyph's bounding box
#[derive(Debug)]
pub enum Corner {
    /// North-East corner
    TopRight,
    /// North-West corner
    TopLeft,
    /// South-East corner
    BottomRight,
    /// South-West corner
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

// TODO (KC): I actually don't understand "kerning" well enough to write good documentation for 
// the following functions. 

/// Computes the amount of kerning between a base glyph and its superscript
pub fn superscript_kern<F : MathFont>(base: &Glyph<F>, script: &Glyph<F>, shift: Unit<FUnit>) -> Unit<FUnit> {
    let base_height = base.bbox.3;
    let script_depth = script.bbox.1 + shift;

    let value1 = kern_from(base, base_height, Corner::TopRight) +
    kern_from(script, base_height, Corner::BottomLeft);

    let value2 = kern_from(base, script_depth, Corner::TopRight) +
    kern_from(script, script_depth, Corner::BottomLeft);

    if value1 > value2 
    { value1 }
    else 
    { value2 }
}

/// Computes the amount of kerning between a base glyph and its subscript
pub fn subscript_kern<F : MathFont>(base: &Glyph<F>, script: &Glyph<F>, shift: Unit<FUnit>) -> Unit<FUnit> {
    let base_depth = base.bbox.1;
    let script_height = script.bbox.3 - shift;

    let value1 = kern_from(base, base_depth, Corner::BottomRight) +
    kern_from(script, base_depth, Corner::TopLeft);

    let value2 = kern_from(base, script_height, Corner::BottomRight) +
    kern_from(script, script_height, Corner::TopLeft);

    if value1 < value2 
    { value1 }
    else 
    { value2 }
}

fn kern_from<F : MathFont>(glyph: &Glyph<F>, height: Unit<FUnit>, side: Corner) -> Unit<FUnit> {
    glyph.font.kern_for(glyph.gid, height, side).unwrap_or(Unit::ZERO)
}