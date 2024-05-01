//! Length constants for layout

use crate::dimensions::{units::{Em, Pt}, Unit};

// From [https://tex.stackexchange.com/questions/48276/latex-specify-font-point-size] & `info latex`
/// A line or an array row's default total height (depth + height).  
/// "A rule of thumb is that the baselineskip should be 1.2 times the font size."
pub const BASELINE_SKIP : Unit<Em> = Unit::<Em>::new(1.2);


/// Additional line height added to [`BASELINE_SKIP`] in `\begin{aligned}` environment.
pub const JOT : f64 = 0.25;


// The values below are gathered from the definition of the corresponding commands in "article.cls" on a default LateX installation
/// For a row in an array, corresponds to the fraction of the row's height (~ [`BASELINE_SKIP`]) which is above the baseline on which characters sit.
pub const STRUT_HEIGHT      : f64 = 0.7;         // \strutbox height = 0.7\baseline
/// For a row in an array, corresponds to the fraction of the row's height (~ [`BASELINE_SKIP`]) which is below the baseline on which characters sit.
pub const STRUT_DEPTH       : f64 = 0.3;         // \strutbox depth  = 0.3\baseline

/// Half the space between two columns of an array, corresponds to LaTeX (`\arraycolsep`)
/// > "\arraycolsep : Half the width of the default horizontal space between columns in an array environment"
/// From Lamport - LateX a document preparation system (2end edition) - p. 207  
pub const COLUMN_SEP        : Unit<Pt> = Unit::<Pt>::new(5.0) ;  // \arraycolsep

/// Width of the vertical bar that separates columns
pub const RULE_WIDTH        : Unit<Pt> = Unit::<Pt>::new(0.4) ;  // \arrayrulewidth

/// Space between two consecutive vertical bars in an array (e.g. `\begin{array}{c||c} .. \end{array}`)
pub const DOUBLE_RULE_SEP   : Unit<Pt> = Unit::<Pt>::new(2.0) ;  // \doublerulesep
