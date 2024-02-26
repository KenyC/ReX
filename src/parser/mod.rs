//! Parses strings representing LateX formulas into [`ParseNode`]'s
//! 
//! The main function function of interest is [`engine::parse`]

pub mod nodes;
pub mod color;
pub mod symbols;
pub mod macros;
pub mod error;
mod textoken;
mod control_sequence;

use unicode_math::AtomType;

use crate::error::ParseResult;
use crate::parser::textoken::InputProcessor;
use crate::parser::textoken::TexToken;
use crate::parser::control_sequence::PrimitiveControlSequence;

use self::error::ParseError;
use self::macros::CommandCollection;
pub use self::nodes::ParseNode;
pub use self::nodes::is_symbol;
use self::symbols::Symbol;
use self::textoken::TokenIterator;


/// Contains the internal state of the TeX parser, what's left to parse, and has methods to parse various TeX construct.  
/// Holds a reference to `CommandCollection`, which holds the definition of custom TeX macros defined by the user.
/// When not using custom macros, the parser can be made `'static`.
pub struct Parser<'c, 'input, I : Iterator<Item = TexToken<'input>>> {
    command_collection : & 'c CommandCollection,
    token_iter : I,
}

impl<'c, 'input, I: Iterator<Item = TexToken<'input>>> Parser<'c, 'input, I> {
    pub fn new(command_collection: & 'c CommandCollection, token_iter: I) -> Self { 
        Self { command_collection, token_iter } 
    }

    const EMPTY_COMMAND_COLLECTION : & 'static CommandCollection = &CommandCollection::new();

    pub fn parse(&mut self) -> ParseResult<Vec<ParseNode>> {
        let Self { command_collection, token_iter } = self;
        let mut results = Vec::new();

        while let Some(token) = token_iter.next() {
            match token {
                TexToken::Char('^') | TexToken::Char('_')  => {

                },
                TexToken::Char(codepoint) => {
                    let atom_type = codepoint_atom_type(codepoint).ok_or_else(|| ParseError::UnrecognizedSymbol(codepoint))?;
                    results.push(ParseNode::Symbol(Symbol { codepoint, atom_type }));
                },
                // Here we deal with "primitive" control sequences, not macros nor environments
                TexToken::ControlSequence(control_sequence_name) => {
                    let command = 
                        PrimitiveControlSequence::from_name(control_sequence_name)
                        .ok_or_else(|| ParseError::UnrecognizedControlSequence(control_sequence_name.to_string().into_boxed_str()))?
                    ;
                    match command {
                        PrimitiveControlSequence::Radical => todo!(),
                        PrimitiveControlSequence::Rule => todo!(),
                        PrimitiveControlSequence::Color => {
                            let color = self.parse_color()?;
                            todo!()
                        },
                        PrimitiveControlSequence::ColorLit(_) => {
                            todo!()
                        },
                        PrimitiveControlSequence::Fraction(_, _, _, _) => todo!(),
                        PrimitiveControlSequence::DelimiterSize(_, _) => todo!(),
                        PrimitiveControlSequence::Kerning(space) => {
                            results.push(ParseNode::Kerning(space))
                        },
                        PrimitiveControlSequence::Style(_) => todo!(),
                        PrimitiveControlSequence::AtomChange(_) => todo!(),
                        PrimitiveControlSequence::TextOperator(_, _) => todo!(),
                        PrimitiveControlSequence::SubStack(_) => todo!(),
                        PrimitiveControlSequence::Text => todo!(),
                    }
                },
            }
        }

        Ok(results)
    }
}


/// This function is the API entry point for parsing tex.
pub fn parse(input: &str) -> ParseResult<Vec<ParseNode>> {
    parse_with_custom_commands(input, &CommandCollection::default())
}


pub fn parse_with_custom_commands<'a>(input: & 'a str, custom_commands : &CommandCollection) -> ParseResult<Vec<ParseNode>> {
    let input_processor = InputProcessor::new(input);
    let mut token_iter = input_processor.token_iter();
    Parser::new(custom_commands, token_iter).parse()
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
