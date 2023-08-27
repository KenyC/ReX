//! Defines structs and parses TeX environments, e.g. `\begin{center}..\end{center}`


use crate::parser::{ParseNode, symbols::Symbol, error::ParseError};

use super::{Parser, error::ParseResult};


/// An enumeration of recognized enviornmnets.
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum Environment {
    /// `\begin{array} ... \end{array}`
    Array,
    /// `\begin{matrix} ... \end{matrix}`
    Matrix,
    /// `\begin{pmatrix} ... \end{pmatrix}`
    PMatrix,
    /// `\begin{bmatrix} ... \end{bmatrix}`
    BMatrix,
    /// `\begin{bbmatrix} ... \end{bbmatrix}`
    BbMatrix,
    /// `\begin{vmatrix} ... \end{vmatrix}`
    VMatrix,
    /// `\begin{vvmatrix} ... \end{vvmatrix}`
    VvMatrix,
}


/// The horizontal positioning of an array.  These are parsed as an optional
/// argument for the Array environment. The default value is `Centered` along
/// the x-axis.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ArrayVerticalAlign {
    /// Centered along the x-axis.
    Centered,

    /// Align the top with the baseline.
    Top,

    /// Align the bottom with the baseline.
    Bottom,
}

impl Default for ArrayVerticalAlign {
    fn default() -> ArrayVerticalAlign {
        ArrayVerticalAlign::Centered
    }
}

// TODO: since we use default values, we should make the argument optional?
/// Array column alignent.  These are parsed as a required macro argument
/// for the array enviornment. The default value is `Centered`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ArrayColumnAlign {
    /// Column is centered
    Centered,

    /// Column is left aligned.
    Left,

    /// Column is right aligned.
    Right,
}

impl Default for ArrayColumnAlign {
    fn default() -> ArrayColumnAlign {
        ArrayColumnAlign::Centered
    }
}

/// Formatting options for a single column.  This includes both the horizontal
/// alignment of the column (clr), and optional vertical bar spacers (on the left).
#[derive(Default, Debug, Clone, Copy, PartialEq, Eq)]
pub struct ArraySingleColumnFormatting {
    /// The alignment of the column.  Defaults to Centered.
    pub alignment: ArrayColumnAlign,

    /// The number of vertical marks before column.
    pub n_vertical_bars_after: u8,
}

/// The collection of column formatting for an array.  This includes the vertical
/// alignment for each column in an array along with optional vertical bars
/// positioned to the right of the last column.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ArrayColumnsFormatting {
    /// The formatting specifications for each column
    pub columns: Vec<ArraySingleColumnFormatting>,

    /// The number of vertical marks after the last column.
    pub n_vertical_bars_before: u8,
}

impl ArrayColumnsFormatting {
    /// Returns center formatting for all columns and no marks
    pub fn default_for(n_cols : usize) -> Self {
        Self { 
            columns:    vec![ArraySingleColumnFormatting::default(); n_cols], 
            n_vertical_bars_before: 0, 
        }
    }
}

/// A structure representing a TeX array `\begin{array}...\end{array}` and similar array like environments like `matrix`, `pmatrix` and the like
#[derive(Debug, Clone, PartialEq)]
pub struct Array {
    /// The formatting arguments (clr) for each row.  Default: center.
    pub col_format: ArrayColumnsFormatting,

    /// A collection of rows.  Each row consists of one `Vec<Expression>`.
    pub rows: Vec<Vec<Expression>>,

    /// The left delimiter for the array (optional).
    pub left_delimiter: Option<Symbol>,

    /// The right delimiter for the array (optional).
    pub right_delimiter: Option<Symbol>,
}

type Expression = Vec<ParseNode>;


impl<'i, 'c> Parser<'i, 'c> {
    /// In `\begin{array}{l|c||}, parses the content of the first argument of the array environment, 
    /// which contains alignment information for columns and number of bars
    pub fn parse_col_format(&mut self) -> ParseResult<ArrayColumnsFormatting> {
        let mut columns = Vec::new();

        let mut n_vertical_bars_before = 0;
        self.consume_whitespace();
        while let Some(_) = self.try_parse_char('|') {
            n_vertical_bars_before += 1;
            self.consume_whitespace();
        }

        loop {
            let alignment;
            match self.parse_char() {
                Some('c')   => alignment = ArrayColumnAlign::Centered,
                Some('r')   => alignment = ArrayColumnAlign::Right,
                Some('l')   => alignment = ArrayColumnAlign::Left,
                Some('}')   => { break; }
                Some(token) => return Err(ParseError::UnrecognizedColumnFormat(token)),
                None => return Err(ParseError::UnexpectedEof),
            }
            
            self.consume_whitespace();


            let mut n_vertical_bars_after = 0_u8;
            while self.try_parse_char('|').is_some() {
                self.consume_whitespace();
                n_vertical_bars_after += 1;
            }


            columns.push(ArraySingleColumnFormatting { 
                alignment, 
                n_vertical_bars_after,
            });

        }

        Ok(ArrayColumnsFormatting { columns, n_vertical_bars_before, })
    }
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn array_col_test() {
        let cols = vec![
            ("c", 
            ArrayColumnsFormatting {
                columns : vec![
                    ArraySingleColumnFormatting {alignment : ArrayColumnAlign::Centered, n_vertical_bars_after : 0}
                ], 
                n_vertical_bars_before: 0
            }),
            ("||l", 
            ArrayColumnsFormatting {
                columns : vec![
                    ArraySingleColumnFormatting {alignment : ArrayColumnAlign::Left, n_vertical_bars_after : 0},
                ], 
                n_vertical_bars_before: 2
            }),
            ("||c|l", 
            ArrayColumnsFormatting {
                columns : vec![
                    ArraySingleColumnFormatting {alignment : ArrayColumnAlign::Centered, n_vertical_bars_after : 1},
                    ArraySingleColumnFormatting {alignment : ArrayColumnAlign::Left,     n_vertical_bars_after : 0},
                ], 
                n_vertical_bars_before: 2
            }),
            ("|  |c l|l|", 
            ArrayColumnsFormatting {
                columns : vec![
                    ArraySingleColumnFormatting {alignment : ArrayColumnAlign::Centered, n_vertical_bars_after : 0},
                    ArraySingleColumnFormatting {alignment : ArrayColumnAlign::Left,     n_vertical_bars_after : 1},
                    ArraySingleColumnFormatting {alignment : ArrayColumnAlign::Left,     n_vertical_bars_after : 1},
                ], 
                n_vertical_bars_before: 2
            }),
            (" |  r| l|| | |r||  ", 
            ArrayColumnsFormatting {
                columns : vec![
                    ArraySingleColumnFormatting {alignment : ArrayColumnAlign::Right, n_vertical_bars_after : 1},
                    ArraySingleColumnFormatting {alignment : ArrayColumnAlign::Left,  n_vertical_bars_after : 4},
                    ArraySingleColumnFormatting {alignment : ArrayColumnAlign::Right, n_vertical_bars_after : 2},
                ], 
                n_vertical_bars_before: 1
            }),
        ];

        for (string, col_format) in cols {
            let mut string = string.to_string();
            string.push('}');

            let mut parser = Parser::new(&string);
            // let mut lexer  = Lexer::new(&string);
            // let style = Style::new();

            assert_eq!(
                parser.parse_col_format().unwrap(),
                col_format,
            );

        }
        
    }
}
