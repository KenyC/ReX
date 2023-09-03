//! Defines structs and parses TeX environments, e.g. `\begin{center}..\end{center}`


use std::fmt::Display;

use crate::parser::{ParseNode, symbols::Symbol, error::ParseError};

use super::{Parser, error::ParseResult, ParseDelimiter};


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

impl Display for Environment {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Environment::Array    => f.write_str("array"),
            Environment::Matrix   => f.write_str("matrix"),
            Environment::PMatrix  => f.write_str("pmatrix"),
            Environment::BMatrix  => f.write_str("bmatrix"),
            Environment::BbMatrix => f.write_str("bbmatrix"),
            Environment::VMatrix  => f.write_str("vmatrix"),
            Environment::VvMatrix => f.write_str("vvmatrix"),
        }
    }
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
    /// Parses a column format string like `l|c||`, as found in e.g. `\begin{array}{l|c||}`. 
    /// This method expects the whole input to be a column format string and will consume the parser.
    pub fn parse_col_format(mut self) -> ParseResult<ArrayColumnsFormatting> {
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
                Some(token) => return Err(ParseError::UnrecognizedColumnFormat(token)),
                None        => { break; },
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

    /// Parses an environment name enclosed with braces, e.g. {envname}
    pub fn parse_environment_name(&mut self) -> ParseResult<Environment> {
        match self.parse_group_as_string() {
            Some("array")    => Ok(Environment::Array),    
            Some("matrix")   => Ok(Environment::Matrix),   
            Some("pmatrix")  => Ok(Environment::PMatrix),  
            Some("bmatrix")  => Ok(Environment::BMatrix),  
            Some("bbmatrix") => Ok(Environment::BbMatrix), 
            Some("vmatrix")  => Ok(Environment::VMatrix),  
            Some("vvmatrix") => Ok(Environment::VvMatrix), 
            Some(name)    => Err(ParseError::UnrecognizedEnvironment(name.to_string())),
            // TODO: that isn't the right error to propagate : failure to parse a group could because we're missing the end delimiter
            None => Err(ParseError::ExpectedOpenGroup),
        }
    }

    fn parse_array_env_body(&mut self, env : Environment) -> ParseResult<Vec<Vec<Expression>>> {
        let mut rows = Vec::new();
        let mut row = Vec::new();
        loop {
            let mut new_parser = self.fork();
            let delimiter = new_parser.parse_expression()?;
            self.input = new_parser.input;
            let results = new_parser.to_results();
            
            match delimiter {
                ParseDelimiter::Alignment => row.push(results),
                ParseDelimiter::EndOfLine => {
                    row.push(results);
                    rows.push(std::mem::take(&mut row));
                },
                ParseDelimiter::EndEnv(name) if name == env => {
                    if !row.is_empty() || !results.is_empty()  { 
                        // An end of line not followed by anything at the end of the environment is (bizarrely) not treated as an empty row in LaTeX
                        row.push(results);
                        rows.push(std::mem::take(&mut row));
                    }
                    break;
                },
                other => return Err(ParseError::ExpectedDelimiter { found: other, expected: ParseDelimiter::EndEnv(env) }),
            }
        }
        Ok(rows)
    }


    /// Parses an environment from the name of the environment to its '\end{env}` command
    pub fn parse_env(&mut self) -> ParseResult<Array> {
        let env = self.parse_environment_name()?;

        let left_delimiter;
        let right_delimiter;
        let mut col_format = None;

        #[inline]
        fn make_delim(character : char) -> Symbol {
            Symbol { codepoint : character, atom_type : unicode_math::AtomType::Inner  }
        }

        match env {
            Environment::Array    => {
                left_delimiter  = None;
                right_delimiter = None;
                // FIXME: ad hoc, should rely on a group routine
                // we're exploiting a particular fact about column format instead of mimicking the logic of the TeX notion of group
                self.consume_whitespace();
                let col_format_string = self.parse_group_as_string().ok_or_else(|| todo!())?; // if not followed by an open gruop we interpret the next char as a column format
                let mut parser = Parser::new(col_format_string);
                parser.input = col_format_string;
                col_format = Some(parser.parse_col_format()?);
            },
            Environment::Matrix   => {
                left_delimiter  = None;
                right_delimiter = None;
            },
            Environment::PMatrix  => {
                left_delimiter  = Some(make_delim('('));
                right_delimiter = Some(make_delim(')'));
            },
            Environment::BMatrix  => {
                left_delimiter  = Some(make_delim('['));
                right_delimiter = Some(make_delim(']'));
            },
            Environment::BbMatrix => {
                left_delimiter  = Some(make_delim('{'));
                right_delimiter = Some(make_delim('}'));
            },
            Environment::VMatrix  => {
                left_delimiter  = Some(make_delim('|'));
                right_delimiter = Some(make_delim('|'));
            },
            Environment::VvMatrix => {
                left_delimiter  = Some(make_delim('‖'));
                right_delimiter = Some(make_delim('‖'));
            },
        }

        let rows = self.parse_array_env_body(env)?;

        let col_format = col_format.unwrap_or_else(|| {
            let n_cols = rows.get(0).map_or(0, Vec::len);
            ArrayColumnsFormatting::default_for(n_cols)
        });

        Ok(Array {col_format, rows, left_delimiter, right_delimiter,})
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
            let mut parser = Parser::new(string);

            assert_eq!(
                parser.parse_col_format().unwrap(),
                col_format,
            );

        }
        
    }


    #[test]
    fn test_parse_array_body() {
        let successes = vec![
            r"1&2\\3&4\end{array}",
            r"1&2\\3&4\\\end{array}",
            r"1&\end{array}",
            r"1&\\\end{array}",
            r"&\\&2\end{array}",
            r"\\\end{array}",
            r"\end{array}",
        ];
        
        for success in successes {
            eprintln!("{}", success);
            let mut parser = Parser::new(success);
            parser.parse_array_env_body(Environment::Array).unwrap();
        }
    }
}
