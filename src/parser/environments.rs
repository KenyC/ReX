use unicode_math::AtomType;

use crate::parser::error::ParseError;
use crate::parser::{tokens_as_string, List};

use super::nodes::{Array, ArrayColumnAlign, ArrayColumnsFormatting, ColSeparator, DummyNode};
use super::symbols::Symbol;
use super::{error::ParseResult, nodes::CellContent, textoken::TexToken, Parser};
use super::{GroupKind, ParseNode};

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
    Aligned,
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
            "aligned"  => Some(Self::Aligned),
            _ => None
        }
    }
}


impl<'a, I : Iterator<Item = TexToken<'a>>> Parser<'a, I> {
    pub fn parse_environment(&mut self, env : Environment) -> ParseResult<Array> {
        let mut col_format = None;

        if let Environment::Array = env {
            let group = self.token_iter
                .capture_group()
                .map_err(|e| match e {
                    ParseError::ExpectedToken => ParseError::MissingColFormatForArrayEnvironment,
                    _ => e,
                })?;

            let mut forked_parser = Parser::from_iter(Self::EMPTY_COMMAND_COLLECTION, group.into_iter());
            col_format = Some(forked_parser.tokens_as_column_format()?);
        }
        let mut rows = self.parse_array_body(env)?;

        let left_delimiter;
        let right_delimiter;

        match env {
            Environment::Array   |
            Environment::Matrix  | 
            Environment::Aligned
            => {
                left_delimiter  = None;
                right_delimiter = None;
            },
            Environment::PMatrix  => {
                left_delimiter  = Some(Symbol {codepoint : '(', atom_type : AtomType::Inner});
                right_delimiter = Some(Symbol {codepoint : ')', atom_type : AtomType::Inner});
            },
            Environment::BMatrix  => {
                left_delimiter  = Some(Symbol {codepoint : '[', atom_type : AtomType::Inner});
                right_delimiter = Some(Symbol {codepoint : ']', atom_type : AtomType::Inner});
            },
            Environment::BbMatrix => {
                left_delimiter  = Some(Symbol {codepoint : '{', atom_type : AtomType::Inner});
                right_delimiter = Some(Symbol {codepoint : '}', atom_type : AtomType::Inner});
            },
            Environment::VMatrix  => {
                left_delimiter  = Some(Symbol {codepoint : '|', atom_type : AtomType::Inner});
                right_delimiter = Some(Symbol {codepoint : '|', atom_type : AtomType::Inner});
            },
            Environment::VvMatrix => {
                left_delimiter  = Some(Symbol {codepoint : '\u{2016}', atom_type : AtomType::Inner});
                right_delimiter = Some(Symbol {codepoint : '\u{2016}', atom_type : AtomType::Inner});
            },
        }

        // For the `aligned` ennvironment, we add dummies in even columns (second, fourth, etc.)
        // which copy the atom_type of the last node of the previous column
        if let Environment::Aligned = env {
            for row in rows.iter_mut() {
                for cell in row.chunks_exact_mut(2) {
                    let atom_type = cell[0].last().map_or_else(
                        || AtomType::Ordinary, 
                        |node| node.atom_type(),
                    );
                    cell[1].insert(0, ParseNode::DummyNode(DummyNode { at: atom_type }));
                }
            }
        }

        let col_format = col_format.unwrap_or_else(|| {
            let n_cols = rows.last().map_or(0, |row| row.len());
            if let Environment::Aligned = env {
                ArrayColumnsFormatting {
                    alignment:  [ArrayColumnAlign::Right, ArrayColumnAlign::Left].iter().cycle().cloned().take(n_cols).collect(),
                    separators: [Vec::new(), vec![ColSeparator::AtExpression(Vec::new())]].iter().cycle().cloned().take(n_cols + 1).collect(),
                }
            }
            else {
                ArrayColumnsFormatting { 
                    alignment:  vec![ArrayColumnAlign::Centered; n_cols], 
                    separators: vec![vec![]; n_cols + 1], 
                }
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

impl<'a, I : Iterator<Item = TexToken<'a>>> Parser<'a, I> {
    fn tokens_as_column_format(&mut self) -> ParseResult<ArrayColumnsFormatting> {
        let mut n_vertical_bars_before = 0;
        let mut current_vertical_bars = &mut n_vertical_bars_before;
        let mut alignment  = Vec::new();
        let mut separators = vec![Vec::new()];
        while let Some(token) = self.token_iter.next_token()? {
            match token {
                  TexToken::Char(c@'c') 
                | TexToken::Char(c@'l') 
                | TexToken::Char(c@'r') 
                => {
                    alignment.push(match c {
                        'c' => ArrayColumnAlign::Centered,
                        'l' => ArrayColumnAlign::Left,
                        'r' => ArrayColumnAlign::Right,
                        _   => unreachable!(), // This has already been ruled out in the previous match
                    });
                    separators.push(Vec::new());
                },
                TexToken::Char('|') 
                => {
                    // Safe to unwrap b/c `separators` always has at least one element
                    let current_separators = separators.last_mut().unwrap();

                    match current_separators.last_mut() {
                        Some(ColSeparator::VerticalBars(bars)) => {
                            *bars += 1;
                        },
                        _ => {
                            current_separators.push(ColSeparator::VerticalBars(1));
                        },
                    }
                },
                TexToken::Char('@') => {
                    let nodes = self.parse_control_seq_argument_as_nodes("@")?;
                    // Safe to unwrap b/c `separators` always has at least one element
                    let current_separators = separators.last_mut().unwrap();
                    current_separators.push(ColSeparator::AtExpression(nodes));
                }
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
            alignment,
            separators,
        })
    }
}

#[cfg(test)]
mod tests {
    use crate::parser::{macros::CommandCollection, textoken::TokenIterator};

    use super::*;

    #[test]
    fn parse_col_format() {
        let cols = vec![
            ("c", 
            ArrayColumnsFormatting {
                alignment  : vec![ArrayColumnAlign::Centered], 
                separators : vec![vec![], vec![]],
            }),
            ("||l", 
            ArrayColumnsFormatting {
                alignment  : vec![ArrayColumnAlign::Left], 
                separators : vec![vec![ColSeparator::VerticalBars(2)], vec![]],
            }),
            ("||c|l", 
            ArrayColumnsFormatting {
                alignment  : vec![ArrayColumnAlign::Centered, ArrayColumnAlign::Left], 
                separators : vec![vec![ColSeparator::VerticalBars(2)], vec![ColSeparator::VerticalBars(1)], vec![]],
            }),
            ("|  |c l|l|", 
            ArrayColumnsFormatting {
                alignment  : vec![
                    ArrayColumnAlign::Centered, 
                    ArrayColumnAlign::Left, 
                    ArrayColumnAlign::Left
                ], 
                separators : vec![
                    vec![ColSeparator::VerticalBars(2)], 
                    vec![],
                    vec![ColSeparator::VerticalBars(1)], 
                    vec![ColSeparator::VerticalBars(1)], 
                ],
            }),
            (" |  r| l|| | |r||  ", 
            ArrayColumnsFormatting {
                alignment  : vec![
                    ArrayColumnAlign::Right, 
                    ArrayColumnAlign::Left, 
                    ArrayColumnAlign::Right,
                ], 
                separators : vec![
                    vec![ColSeparator::VerticalBars(1)], 
                    vec![ColSeparator::VerticalBars(1)], 
                    vec![ColSeparator::VerticalBars(4)], 
                    vec![ColSeparator::VerticalBars(2)], 
                ],
            }),
        ];

        for (string, col_format) in cols {
            let command_collection = CommandCollection::new();
            let mut parser = Parser::new(&command_collection, string);

            assert_eq!(
                parser.tokens_as_column_format(),
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