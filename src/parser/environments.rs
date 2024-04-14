use crate::parser::error::ParseError;
use crate::parser::{tokens_as_string, List};

use super::nodes::{Array, ArrayColumnAlign, ArrayColumnsFormatting, ArraySingleColumnFormatting};
use super::symbols::Symbol;
use super::{error::ParseResult, nodes::CellContent, textoken::TexToken, Parser};
use super::GroupKind;

/// An enumeration of recognized enviornmnets.
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum Environment {
    Array,
    Matrix,
    PMatrix,
    BMatrix,
    BbMatrix,
    VMatrix,
    VvMatrix,
}

impl Environment {
    pub fn from_name(name : &str) ->  Option<Self> {
        match name {
            "array"    => Some(Self::Array),
            "matrix"   => Some(Self::Matrix),
            "pmatrix"  => Some(Self::PMatrix),
            "bmatrix"  => Some(Self::BMatrix),
            "Bmatrix"  => Some(Self::BbMatrix),
            "vmatrix"  => Some(Self::VMatrix),
            "Vmatrix"  => Some(Self::VvMatrix),
            _ => None
        }
    }
}


impl<'a, I : Iterator<Item = TexToken<'a>>> Parser<'a, I> {
    pub fn parse_environment(&mut self, env : Environment) -> ParseResult<Array> {
        let mut col_format = None;

        if let Environment::Array = env {
            let tokens = self.token_iter
                .capture_group()
                .map_err(|e| match e {
                    ParseError::ExpectedToken => ParseError::MissingColFormatForArrayEnvironment,
                    _ => e,
                })?;
            col_format = Some(tokens_as_column_format(tokens.into_iter())?);
        }
        let rows = self.parse_array_body(env)?;

        let left_delimiter;
        let right_delimiter;

        match env {
            Environment::Array    => {
                left_delimiter  = None;
                right_delimiter = None;
            },
            Environment::Matrix   => {
                left_delimiter  = None;
                right_delimiter = None;
            },
            Environment::PMatrix  => {
                left_delimiter  = Some(Symbol {codepoint : '(', atom_type : unicode_math::AtomType::Inner});
                right_delimiter = Some(Symbol {codepoint : ')', atom_type : unicode_math::AtomType::Inner});
            },
            Environment::BMatrix  => {
                left_delimiter  = Some(Symbol {codepoint : '[', atom_type : unicode_math::AtomType::Inner});
                right_delimiter = Some(Symbol {codepoint : ']', atom_type : unicode_math::AtomType::Inner});
            },
            Environment::BbMatrix => {
                left_delimiter  = Some(Symbol {codepoint : '{', atom_type : unicode_math::AtomType::Inner});
                right_delimiter = Some(Symbol {codepoint : '}', atom_type : unicode_math::AtomType::Inner});
            },
            Environment::VMatrix  => {
                left_delimiter  = Some(Symbol {codepoint : '|', atom_type : unicode_math::AtomType::Inner});
                right_delimiter = Some(Symbol {codepoint : '|', atom_type : unicode_math::AtomType::Inner});
            },
            Environment::VvMatrix => {
                left_delimiter  = Some(Symbol {codepoint : '\u{2016}', atom_type : unicode_math::AtomType::Inner});
                right_delimiter = Some(Symbol {codepoint : '\u{2016}', atom_type : unicode_math::AtomType::Inner});
            },
        }

        let col_format = col_format.unwrap_or_else(|| {
            let n_cols = rows.last().map_or(0, |row| row.len());
            ArrayColumnsFormatting {
                n_vertical_bars_before: 0,
                columns: vec![ArraySingleColumnFormatting { 
                    alignment: ArrayColumnAlign::Centered, 
                    n_vertical_bars_after: 0, 
                }; n_cols],
            }
        });

        Ok(Array {
            col_format,
            rows,
            left_delimiter,
            right_delimiter,
        })
    }


    pub fn parse_array_body(&mut self, env : Environment) -> ParseResult<Vec<Vec<CellContent>>> {
        let mut to_return    = Vec::new();
        let mut current_line = Vec::new();

        while {
            let List {nodes, group} = self.parse_until_end_of_group()?;

            match group {
                GroupKind::Env(env_ended) if env == env_ended => {
                    if !current_line.is_empty() || !nodes.is_empty() {
                        current_line.push(nodes);
                        to_return.push(std::mem::take(&mut current_line));
                    }
                    false
                },
                GroupKind::Align => {
                    current_line.push(nodes);
                    true
                },
                GroupKind::NewLine => {
                    current_line.push(nodes);
                    to_return.push(std::mem::take(&mut current_line));
                    true
                },

                _ => return Err(ParseError::UnexpectedEndGroup { expected : vec![GroupKind::Align, GroupKind::NewLine, GroupKind::Env(env)].into_boxed_slice(), got : group }),
            }
        }
        {}


        Ok(to_return)
    }
}

fn tokens_as_column_format<'a, I : Iterator<Item = TexToken<'a>>>(iterator : I) -> ParseResult<ArrayColumnsFormatting> {
    let mut n_vertical_bars_before = 0;
    let mut current_vertical_bars = &mut n_vertical_bars_before;
    let mut columns = Vec::new();
    for token in iterator {
        match token {
              TexToken::Char(c@'c') 
            | TexToken::Char(c@'l') 
            | TexToken::Char(c@'r') 
            => {
                columns.push(ArraySingleColumnFormatting {
                    alignment: match c {
                        'c' => ArrayColumnAlign::Centered,
                        'l' => ArrayColumnAlign::Left,
                        'r' => ArrayColumnAlign::Right,
                        _   => unreachable!(), // This has already been ruled out in the previous match
                    },
                    n_vertical_bars_after: 0,
                });
                current_vertical_bars = &mut columns.last_mut().unwrap_or_else(|| unreachable!()).n_vertical_bars_after;
            },
            TexToken::Char('|') 
            => {
                *current_vertical_bars += 1;
            },
            TexToken::Char(_) => {
                return Err(ParseError::UnrecognizedArrayColumnFormat);
            }
            TexToken::WhiteSpace => (),

            TexToken::BeginGroup 
            | TexToken::EndGroup 
            | TexToken::ControlSequence(_) 
            | TexToken::Superscript 
            | TexToken::Prime { .. } 
            | TexToken::Alignment 
            | TexToken::Subscript => return Err(ParseError::UnrecognizedArrayColumnFormat),
        }
    }
    Ok(ArrayColumnsFormatting {
        columns,
        n_vertical_bars_before,
    })
}


#[cfg(test)]
mod tests {
    use crate::parser::textoken::TokenIterator;

    use super::*;

    #[test]
    fn parse_col_format() {
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
            let mut token_iter = TokenIterator::new(string);

            assert_eq!(
                tokens_as_column_format(token_iter),
                Ok(col_format),
            );

        }
        
    }


    #[test]
    fn good_arrays() {
        let collection = crate::parser::macros::CommandCollection::default();
        let mut parser = Parser::new(&collection, r"1&2\\3&4\end{pmatrix}");
        let result = parser.parse_environment(Environment::PMatrix);
        result.unwrap();
    }
}