//! Parses strings representing LateX formulas into [`ParseNode`]'s
//! 
//! The main function function of interest is [`engine::parse`]

pub mod nodes;
pub mod color;
pub mod symbols;
pub mod macros;
pub mod error;
pub mod environments;
mod textoken;
mod control_sequence;

use unicode_math::AtomType;

use crate::error::ParseResult;
use crate::font::style_symbol;
use crate::font::Style;
use crate::parser::control_sequence::parse_color;
use crate::parser::nodes::GenFraction;
use crate::parser::nodes::PlainText;
use crate::parser::textoken::InputProcessor;
use crate::parser::textoken::TexToken;
use crate::parser::control_sequence::PrimitiveControlSequence;

use self::environments::Environment;
use self::error::ParseError;
use self::macros::CommandCollection;
use self::macros::ExpandedTokenIter;
pub use self::nodes::ParseNode;
pub use self::nodes::is_symbol;
use self::symbols::Symbol;
use self::textoken::TokenIterator;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GroupKind {
    BraceGroup,
    Env(Environment),
    // a group ended by &
    Cell,
    // a group end by \\
    Line,
    // end of file
    EndOfInput,

}

struct List {
    nodes : Vec<ParseNode>,
    group : GroupKind
}


/// Contains the internal state of the TeX parser, what's left to parse, and has methods to parse various TeX construct.  
/// Holds a reference to `CommandCollection`, which holds the definition of custom TeX macros defined by the user.
/// When not using custom macros, the parser can be made `'static`.
pub struct Parser<'a, I : Iterator<Item = TexToken<'a>>> {
    token_iter : ExpandedTokenIter<'a, I>,
    current_style : Style,
}

impl<'a> Parser<'a, TokenIterator<'a>> {
    pub fn new<'command : 'a, 'input : 'a>(command_collection: & 'command CommandCollection, input: & 'input str) -> Self { 
        Self { 
            token_iter : ExpandedTokenIter::new(command_collection, TokenIterator::new(input)),
            current_style : Style::default()
        } 
    }
}

impl<'a, I : Iterator<Item = TexToken<'a>>> Parser<'a, I> {

    const EMPTY_COMMAND_COLLECTION : & 'static CommandCollection = &CommandCollection::new();

    pub fn from_iter<'command : 'a>(command_collection: & 'command CommandCollection, input: I) -> Self { 
        Self { 
            token_iter : ExpandedTokenIter::new(command_collection, input),
            current_style: Style::default(),
        } 
    }

    pub fn parse(&mut self) -> ParseResult<Vec<ParseNode>> {
        let List { nodes, group } = self.parse_until_end_of_group()?;
        if let GroupKind::EndOfInput = group 
        { Ok(nodes) }
        else 
        { Err(ParseError::UnexpectedEndGroup(group)) }
    }


    fn parse_until_end_of_group(&mut self) -> ParseResult<List> {
        let mut results = Vec::new();

        while let Some(token) = self.token_iter.next_token()? {
            match token {
                TexToken::Char('^') | TexToken::Char('_')  => {

                },
                _ if token.is_end_group() => {
                    return Ok(List { nodes: results, group: GroupKind::BraceGroup });
                },
                TexToken::Char(codepoint) => {
                    let atom_type = codepoint_atom_type(codepoint).ok_or_else(|| ParseError::UnrecognizedSymbol(codepoint))?;
                    let codepoint = style_symbol(codepoint, self.current_style);
                    results.push(ParseNode::Symbol(Symbol { codepoint, atom_type }));
                },
                // Here we deal with "primitive" control sequences, not macros
                TexToken::ControlSequence(control_sequence_name) => {
                    let command = 
                        PrimitiveControlSequence::from_name(control_sequence_name)
                        .ok_or_else(|| ParseError::UnrecognizedControlSequence(control_sequence_name.to_string().into_boxed_str()))?
                    ;
                    use PrimitiveControlSequence::*;
                    match command {
                        Radical => {
                            let inner = self.parse_required_control_seq_argument_as_nodes(control_sequence_name)?;
                            results.push(ParseNode::Radical(nodes::Radical { inner, }));
                        },
                        Rule => todo!(),
                        Color => {
                            let color_name_group = self.token_iter.capture_group()?;
                            let color = parse_color(color_name_group.into_iter())?;
                            let inner = self.parse_required_control_seq_argument_as_nodes(control_sequence_name)?;
                            results.push(ParseNode::Color(nodes::Color {
                                color,
                                inner,
                            }));
                        },
                        ColorLit(color) => {
                            let inner = self.parse_required_control_seq_argument_as_nodes(control_sequence_name)?;
                            results.push(ParseNode::Color(nodes::Color {
                                color,
                                inner,
                            }));
                        },
                        Fraction(left_delimiter, right_delimiter, bar_thickness, style) => {
                            let numerator   = self.parse_required_control_seq_argument_as_nodes(control_sequence_name)?;
                            let denominator = self.parse_required_control_seq_argument_as_nodes(control_sequence_name)?;

                            results.push(ParseNode::GenFraction(GenFraction {
                                numerator, denominator,
                                left_delimiter, right_delimiter,
                                bar_thickness, style,
                            }));
                        },
                        DelimiterSize(_, _) => todo!(),
                        Kerning(space) => {
                            results.push(ParseNode::Kerning(space))
                        },
                        StyleCommand(_) => todo!(),
                        AtomChange(_) => todo!(),
                        TextOperator(_, _) => todo!(),
                        SubStack(_) => todo!(),
                        Text => {
                            let text_group = self.token_iter.capture_group()?;
                            let text = tokens_as_string(text_group.into_iter())?;
                            results.push(ParseNode::PlainText(PlainText {
                                text,
                            }));
                        },
                        BeginEnv => {
                            let env_name_group = self.token_iter.capture_group()?;
                            let env_name = tokens_as_string(env_name_group.into_iter())?;
                            todo!()
                        },
                        EndEnv => todo!(),
                        SymbolCommand(symbol) => {
                            let Symbol { codepoint, atom_type } = symbol;
                            let codepoint = style_symbol(codepoint, self.current_style);
                            results.push(ParseNode::Symbol(Symbol { codepoint, atom_type, }));
                        },
                    }
                },
            }
        }

        Ok(List { nodes: results, group: GroupKind::EndOfInput })
    }

    fn parse_required_control_seq_argument_as_nodes(&mut self, control_seq_name : &str) -> ParseResult<Vec<ParseNode>> {
        self.parse_required_argument_as_nodes()
            .map_err(|e| match e {
                ParseError::ExpectedToken => ParseError::MissingArgForCommand(Box::from(control_seq_name)),
                e => e,
            })
    }

    fn parse_required_argument_as_nodes(&mut self) -> ParseResult<Vec<ParseNode>> {
        let group = self.token_iter.capture_group()?;

        // Normally all tokens are already expanded after `capture_group`
        // There is no need to have further expansions
        let mut forked_parser = Parser::from_iter(Self::EMPTY_COMMAND_COLLECTION, group.into_iter());
        forked_parser.current_style = self.current_style;

        let List { nodes, group } = forked_parser.parse_until_end_of_group()?;

        if group != GroupKind::EndOfInput {
            return Err(ParseError::UnexpectedEndGroup(group));
        }
        
        Ok(nodes)
    }
}

fn tokens_as_string<'a, I : Iterator<Item = TexToken<'a>>>(iterator : I) -> ParseResult<String> {
    let mut to_return = String::new();
    for token in iterator {
        match token {
            TexToken::Char(c) => to_return.push(c),
            TexToken::ControlSequence(_) => return Err(ParseError::ExpectedChars),
        }
    }
    Ok(to_return)
}


/// This function is the API entry point for parsing tex.
pub fn parse(input: &str) -> ParseResult<Vec<ParseNode>> {
    parse_with_custom_commands(input, &CommandCollection::default())
}


pub fn parse_with_custom_commands<'a>(input: & 'a str, custom_commands : &CommandCollection) -> ParseResult<Vec<ParseNode>> {
    Parser::new(custom_commands, input).parse()
}







/// Helper function for determining an atomtype based on a given codepoint.
/// This is primarily used for characters while processing, so may give false
/// negatives when used for other things.
fn codepoint_atom_type(codepoint: char) -> Option<AtomType> {
    Some(match codepoint {
             'a' ..= 'z' | 'A' ..= 'Z' | '0' ..= '9' | 'Α' ..= 'Ω' | 'α' ..= 'ω' => AtomType::Alpha,
             '*' | '+' | '-' => AtomType::Binary,
             '[' | '(' => AtomType::Open,
             ']' | ')' | '?' | '!' => AtomType::Close,
             '=' | '<' | '>' | ':' => AtomType::Relation,
             ',' | ';' => AtomType::Punctuation,
             '|' => AtomType::Fence,
             '/' | '@' | '.' | '"' => AtomType::Alpha,
             _ => return None,
         })
}



#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn snapshot_symbols() {
        insta::assert_debug_snapshot!(parse("1"));
        insta::assert_debug_snapshot!(parse("a"));
        insta::assert_debug_snapshot!(parse("+"));
        insta::assert_debug_snapshot!(parse(r"\mathrm A"));
        insta::assert_debug_snapshot!(parse(r"\mathfrak A"));
        insta::assert_debug_snapshot!(parse(r"\alpha"));
        // should object to cyrillic characters
        insta::assert_debug_snapshot!(parse(r"Ж"));
    }

    #[test]
    fn snapshot_frac() {
        insta::assert_debug_snapshot!(parse(r"\frac 12"));
        insta::assert_debug_snapshot!(parse(r"\frac{1+0} {2+2}"));
        insta::assert_debug_snapshot!(parse(r"\frac \left(1\right)2"));
        insta::assert_debug_snapshot!(parse(r"\frac\alpha\beta"));
    }

    #[test]
    fn snapshot_radicals() {
        // success
        insta::assert_debug_snapshot!(parse(r"\sqrt{x}"));
        insta::assert_debug_snapshot!(parse(r"\sqrt2"));
        insta::assert_debug_snapshot!(parse(r"\sqrt\alpha"));
        insta::assert_debug_snapshot!(parse(r"1^\sqrt2"));
        insta::assert_debug_snapshot!(parse(r"\alpha_\sqrt{1+2}"));
        insta::assert_debug_snapshot!(parse(r"\sqrt\sqrt2"));
        insta::assert_debug_snapshot!(parse(r"\sqrt2_3" ));
        insta::assert_debug_snapshot!(parse(r"\sqrt{2_3}"));

        // fail
        insta::assert_debug_snapshot!(parse(r"\sqrt" ));
        insta::assert_debug_snapshot!(parse(r"\sqrt_2" ));
        insta::assert_debug_snapshot!(parse(r"\sqrt^2"));
    }


    #[test]
    fn snapshot_scripts() {
        insta::assert_debug_snapshot!(parse(r"1_2"));
        insta::assert_debug_snapshot!(parse(r"1_2^3"));
        insta::assert_debug_snapshot!(parse(r"1^3_2"));
        insta::assert_debug_snapshot!(parse(r"1^\alpha"));
        insta::assert_debug_snapshot!(parse(r"1^2^3"));
        insta::assert_debug_snapshot!(parse(r"1^{2^3}"));
        insta::assert_debug_snapshot!(parse(r"{a^b}_c"));
        insta::assert_debug_snapshot!(parse(r"1_{1+1}^{2+1}"));
    }


    #[test]
    fn snapshot_delimited() {
        // success
        insta::assert_debug_snapshot!(parse(r"\left(\right)"));
        insta::assert_debug_snapshot!(parse(r"\left(\right."));
        insta::assert_debug_snapshot!(parse(r"\left(\alpha\right)"));
        insta::assert_debug_snapshot!(parse(r"\left(\alpha+1\right)"));
        insta::assert_debug_snapshot!(parse(r"\left(1\middle|2\right)"));
        insta::assert_debug_snapshot!(parse(r"\left(1\middle|2\middle|3\right)"));
        insta::assert_debug_snapshot!(parse(r"\left\lBrack{}x\right\rBrack"));

        // fail
        insta::assert_debug_snapshot!(parse(r"\left(1\middle|"));
        insta::assert_debug_snapshot!(parse(r"\right(1+1"));
        insta::assert_debug_snapshot!(parse(r"\left)1+1\right)"));
    }


    #[test]
    fn snapshot_array() {
        insta::assert_debug_snapshot!(parse(r"\begin{array}{c}\end{array}"));
        insta::assert_debug_snapshot!(parse(r"\begin{array}{c}1\\2\end{array}"));
        insta::assert_debug_snapshot!(parse(r"\begin{array}{c}1\\\end{array}"));
        insta::assert_debug_snapshot!(parse(r"\begin{pmatrix}1&2\\3&4\end{pmatrix}"));
        insta::assert_debug_snapshot!(parse(r"\begin{array}{c|l}1&\alpha\\2&\frac12\end{array}"));
        insta::assert_debug_snapshot!(parse(r"\begin{array}{cc}1 \\ 2"));
    }

    #[ignore = "unsupported as of yet"]
    #[test]
    fn snapshot_rule() {
        insta::assert_debug_snapshot!(parse(r"\rule{1cm}{3pt}"));
        insta::assert_debug_snapshot!(parse(r"\rule{4pt}{5px}"));
    }

    #[test]
    fn snapshot_plain_text() {
        insta::assert_debug_snapshot!(parse(r"\text{abc}"));
        insta::assert_debug_snapshot!(parse(r"\text{abc}def"));
        insta::assert_debug_snapshot!(parse(r"\text{\{\}1}1}"));
        insta::assert_debug_snapshot!(parse(r"\text{}}"));
    }

    #[test]
    fn snapshot_color() {
        // success
        insta::assert_debug_snapshot!(parse(r"\color{cyan}{1+1}"));
        insta::assert_debug_snapshot!(parse(r"\color{red}{1+1}"));
        insta::assert_debug_snapshot!(parse(r"\red{1}"));
        insta::assert_debug_snapshot!(parse(r"\blue{1}"));
        insta::assert_debug_snapshot!(parse(r"\gray{1}"));
        insta::assert_debug_snapshot!(parse(r"\color{chartreuse}\alpha"));
        insta::assert_debug_snapshot!(parse(r"\color{chocolate}\alpha"));

        // fail
        insta::assert_debug_snapshot!(parse(r"\color{bred}{1+1}"));
        insta::assert_debug_snapshot!(parse(r"\color{bred}1"));
        insta::assert_debug_snapshot!(parse(r"\color red{1}"));
    }

    #[test]
    fn snapshot_style() {
        // success
        insta::assert_debug_snapshot!(parse(r"1\scriptstyle2"));
        insta::assert_debug_snapshot!(parse(r"{1\scriptstyle}2"));
        insta::assert_debug_snapshot!(parse(r"1\textstyle2"));
        insta::assert_debug_snapshot!(parse(r"1\sqrt{\displaystyle s}1"));
    }


    #[test]
    fn snapshot_atom_change() {
        // success
        insta::assert_debug_snapshot!(parse(r"1\mathrel{R}2"));
        insta::assert_debug_snapshot!(parse(r"1\mathrel{\frac{1}{2}} 2"));
        insta::assert_debug_snapshot!(parse(r"\mathop{1}2"));
    }


    #[test]
    fn snapshot_text_operators() {
        // success
        insta::assert_debug_snapshot!(parse(r"\sin 1"));
        insta::assert_debug_snapshot!(parse(r"\log (42 + 1)"));
        insta::assert_debug_snapshot!(parse(r"\sin(a + b) = \sin a \cos b + \cos b \sin a"));
        insta::assert_debug_snapshot!(parse(r"\det_{B} M"));
        insta::assert_debug_snapshot!(parse(r"\lim_{h \to 0 } \frac{f(x+h)-f(x)}{h}"));
    }


    #[test]
    fn snapshot_spacing() {
        // success
        insta::assert_debug_snapshot!(parse(r"1\!2"));
        insta::assert_debug_snapshot!(parse(r"2\quad 3"));
        insta::assert_debug_snapshot!(parse(r"2\quad3"));
        insta::assert_debug_snapshot!(parse(r"5\,2"));
        insta::assert_debug_snapshot!(parse(r"5\;2"));
        insta::assert_debug_snapshot!(parse(r"5\:2"));
        insta::assert_debug_snapshot!(parse(r"1\qquad{}33"));

        // failure
        insta::assert_debug_snapshot!(parse(r"1\33"));
    }

    #[test]
    fn snapshot_delimiter() {
        // success
        insta::assert_debug_snapshot!(parse(r"\biggl("));
        insta::assert_debug_snapshot!(parse(r"\bigr]"));
        insta::assert_debug_snapshot!(parse(r"\Bigl\langle"));
        insta::assert_debug_snapshot!(parse(r"\Biggr|"));
        insta::assert_debug_snapshot!(parse(r"\Bigl\lBrack"));

        // failure
        insta::assert_debug_snapshot!(parse(r"\bigr\lBrack"));
        insta::assert_debug_snapshot!(parse(r"\Bigl\rangle"));
    }

    #[test]
    fn snapshot_substack() {
        // success
        insta::assert_debug_snapshot!(parse(r"\substack{   1 \\ 2}"));
        insta::assert_debug_snapshot!(parse(r"\substack{ 1 \\ \frac{7}8 \\ 4}"));
        insta::assert_debug_snapshot!(parse(r"\begin{array}{c}\substack{1 \\ \frac{7}8 \\ 4} \\ 5 \end{array}"));
        insta::assert_debug_snapshot!(parse(r"\substack{1 \\}"));
        insta::assert_debug_snapshot!(parse(r"1 \substack{}"));

        // failure
        insta::assert_debug_snapshot!(parse(r"\substack{ 1 \\ 2}\\"));
        insta::assert_debug_snapshot!(parse(r"\substack \alpha \\ 1"));
        insta::assert_debug_snapshot!(parse(r"\substack{ 1 \\ 1"));
    }

    #[test]
    fn snapshot_style_change() {
        // success
        insta::assert_debug_snapshot!(parse(r"1\scriptstyle 2"));
        insta::assert_debug_snapshot!(parse(r"1\scriptstyle2\textstyle1+1"));
        insta::assert_debug_snapshot!(parse(r"1{\scriptstyle2\textstyle1}+1"));
        insta::assert_debug_snapshot!(parse(r"\frac{22\scriptscriptstyle22}2"));
    }


    #[test]
    fn snapshot_primes() {
        insta::assert_debug_snapshot!(parse("a'"));
        insta::assert_debug_snapshot!(parse("a''"));
        insta::assert_debug_snapshot!(parse("a'''"));
        insta::assert_debug_snapshot!(parse("a''''"));
        insta::assert_debug_snapshot!(parse("'a"));
        insta::assert_debug_snapshot!(parse(r"\sqrt'"));
    }
}
