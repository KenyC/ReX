//! Parses strings representing LateX formulas into [`ParseNode`]'s
//! 
//! Defines the [`parse`] function to parse TeX into renderable [`ParseNode`]. 
//! More fine-grained customization is offered by the more basic `Parser` struct, which allows custom commands.



#[deny(missing_docs)]
pub mod nodes;
#[deny(missing_docs)]
pub mod color;
#[deny(missing_docs)]
pub mod symbols;
#[deny(missing_docs)]
pub mod macros;
pub mod environments;
pub mod functions;
pub mod lexer;
pub mod error;

pub use self::nodes::ParseNode;
pub use self::nodes::is_symbol;




use std::todo;

use crate::parser::error::{ParseError, ParseResult};
use crate::font::{Style, style_symbol, AtomType};
use crate::parser::symbols::codepoint_atom_type;
use crate::parser::{
    nodes::{Delimited, Accent, Scripts},
    symbols::Symbol,
    color::RGBA,
    environments::Environment,
};
use crate::parser::functions::{Command};
use crate::parser::macros::CommandCollection;
use crate::dimensions::AnyUnit;


#[derive(Debug, Clone, Copy)]
enum ParseDelimiter {
    CloseBracket
}

/// A parser, contains some input, some parameters of parsing such as custom commands and some parser state info.  
/// The lifetime `'i` is for the borrow of the input.
/// The lifetime `'c` is for the borrow of the custom command collection (cf [`CommandCollection`]).
pub struct Parser<'i, 'c> {
    input : & 'i str,
    local_style : Style,
    stop_parsing_at : Option<ParseDelimiter>,
    custom_commands : Option<& 'c CommandCollection>,
    result : Vec<ParseNode>,
}

impl<'i, 'c> Parser<'i, 'c> {
    /// Creates a new parser from an input string.
    pub fn new(input : & 'i str) -> Self {
        Self { 
            input, 
            local_style: Style::default(), 
            custom_commands: None,
            stop_parsing_at: None,
            result : Vec::new(),
        }
    }

    /// Sets the [`Style`] for the parser.
    pub fn with_style(mut self, style : Style) -> Self {
        self.local_style = style;
        self
    }

    /// Sets a library of custom commands for the parser
    pub fn with_custom_commands(mut self, custom_commands : & 'c CommandCollection) -> Self {
        self.custom_commands = Some(custom_commands);
        self
    }

    /// Parses the input provided into [`ParseNode`]s. This is the main API entry point for parsing.
    pub fn parse(mut self) -> ParseResult<Vec<ParseNode>> {
        self.parse_expression()?;
        Ok(self.result)
    }

    fn parse_expression(&mut self) -> ParseResult<()> {
        while !self.end_of_parse()? {
            self.consume_whitespace();

            // We try to parse the first things that comes along
            let node = 
                self.parse_control_sequence()
                .or_else(|| self.parse_symbol())
            ;


            // We attach sub- super-scripts to it if we can



            // Nothing was recognized
            if node.is_none() {
                return todo!()
            }
        }
        // todo!();
        Ok(())
    }

    /// Recovers the nodes that have been parsed yet, without running any more parsing
    pub fn to_results(self) -> Vec<ParseNode> {
        self.result
    }


    /// Check if parser has reached end of parse.   
    /// This maybe because we've hit end of input or if `Parser::stop_parsing_at` is set, because we've reached a delimiter. 
    fn end_of_parse(&mut self) -> ParseResult<bool> {
        let is_empty = self.input.is_empty();
        Ok(match self.stop_parsing_at {
            None => is_empty,
            _ if is_empty => return Err(ParseError::UnexpectedEof),
            Some(ParseDelimiter::CloseBracket) => self.try_parse_char('}').is_some(),
        })
    }


    /// Expects to parse something of the form `\foo[..]{..}` with correct number of arguments and well-typed arguments
    /// If unable to, it does not advance input.
    fn parse_control_sequence(&mut self) -> Option<ParseResult<ParseNode>> {
        let Self { input, result, .. } = self;
        let control_seq_name = self.control_sequence()?;

            
        // When the command name has been recognized, we are sure this is a command
        // Any failure from now on must be a parsing error (misformed input)
        Some(
            // First case, a TeX command
            if let Some(command) = Command::from_name(control_seq_name) {
                match command {
                    Command::Radical => self.parse_radical().map(|radical| ParseNode::Radical(radical)),
                    Command::Rule => todo!(),
                    Command::Color => todo!(),
                    Command::ColorLit(_) => todo!(),
                    Command::Fraction(_, _, _, _) => todo!(),
                    Command::DelimiterSize(_, _) => todo!(),
                    Command::Kerning(_) => todo!(),
                    Command::Style(_) => todo!(),
                    Command::AtomChange(_) => todo!(),
                    Command::TextOperator(_, _) => todo!(),
                    Command::SubStack(_) => todo!(),
                    Command::Text => todo!(),
                }
            }
            // second case: a symbol with a name
            else if let Some(symbol) = Symbol::from_name(control_seq_name) {
                Ok(ParseNode::Symbol(symbol))
            }
            // third case: a custom macro
            else if todo!() {
                todo!()
            }
            else {
                Err(todo!())
            }
        )

    }


    /// Parses a symbol
    fn parse_symbol(&mut self) -> Option<ParseResult<ParseNode>> {
        let codepoint = self.parse_char()?;
        // This baroque closure construction allows us to use `?` to propagate an error to the ParseResult
        // Otherwise, the `?` would propagate to `Option`.
        Some((|| {
            let atom_type = codepoint_atom_type(codepoint).ok_or_else(|| todo!())?;
            Ok(ParseNode::Symbol(Symbol { codepoint, atom_type, }))
        })())
    }

    /// Creates a new parser in the same state, with no nodes
    fn fork(&self) -> Self {
    	let Self { input, local_style, custom_commands, stop_parsing_at, .. } = self;
    	Self { 
    		input, 
    		local_style : local_style.clone(), 
    		custom_commands : *custom_commands, 
            stop_parsing_at : *stop_parsing_at,
    		result: Vec::new(),
    	}
    }

    fn parse_group(&mut self) -> Option<ParseResult<Vec<ParseNode>>> {
        let mut parser = self.fork();

        parser.try_parse_char('{')?;

        parser.stop_parsing_at = Some(ParseDelimiter::CloseBracket);
        let result = parser.parse_expression();
        if let Err(e) = result {
            return Some(Err(e));
        }

        self.input = parser.input;

        Some(Ok(parser.to_results()))
    }
}


/// This function is the API entry point for parsing tex.
pub fn parse(input: &str) -> ParseResult<Vec<ParseNode>> {
    Parser::new(input).parse()
}



// --------------
//     TESTS
// --------------

#[cfg(test)]
mod tests {
    use std::eprintln;

    use crate::parser::{parse, Parser, macros::{CustomCommand, CommandCollection}, ParseNode, nodes::PlainText, error::{ParseResult, ParseError}, ParseDelimiter};


    #[test]
    fn planck_h() {
        let mut errs: Vec<String> = Vec::new();
        should_pass!(errs, parse, [r"h"]);
        display_errors!(errs);
    }

    #[test]
    fn ldots() {
        let mut errs: Vec<String> = Vec::new();
        should_pass!(errs, parse, [r"\ldots",r"\vdots",r"\dots"]);
        display_errors!(errs);
    }

    #[test]
    fn fractions() {
        let mut errs: Vec<String> = Vec::new();
        should_pass!(errs, parse, [r"\frac\alpha\beta", r"\frac\int2"]);
        should_fail!(errs, parse, [r"\frac \left(1 + 2\right) 3"]);
        should_equate!(errs,
                       parse,
                       [(r"\frac12", r"\frac{1}{2}"),
                        (r"\frac \sqrt2 3", r"\frac{\sqrt2}{3}"),
                        (r"\frac \frac 1 2 3", r"\frac{\frac12}{3}"),
                        (r"\frac 1 \sqrt2", r"\frac{1}{\sqrt2}")]);
        display_errors!(errs);
    }

    #[test]
    fn radicals() {
        let mut errs: Vec<String> = Vec::new();
        // TODO: Add optional paramaters for radicals
        let cases = vec![
            r"\sqrt{x}",
            r"\sqrt2",
            r"\sqrt\alpha",
            r"1^\sqrt2",
            r"\alpha_\sqrt{1+2}",
            r"\sqrt\sqrt2"
        ];
        for case in cases {
            eprintln!("{}", case);
            let result = parse(case);
            result.unwrap();
        }

        should_fail!(errs, parse, [r"\sqrt", r"\sqrt_2", r"\sqrt^2"]);
        should_equate!(errs, parse, [(r"\sqrt2", r"\sqrt{2}")]);
        should_differ!(errs, parse, [(r"\sqrt2_3", r"\sqrt{2_3}")]);
        display_errors!(errs);
    }

    #[test]
    fn scripts() {
        let mut errs: Vec<String> = Vec::new();
        should_pass!(errs,
                     parse,
                     [r"1_2^3",
                      r"_1",
                      r"^\alpha",
                      r"_2^\alpha",
                      r"1_\frac12",
                      r"2^\alpha",
                      r"x_{1+2}",
                      r"x^{2+3}",
                      r"x^{1+2}_{2+3}",
                      r"a^{b^c}",
                      r"{a^b}^c",
                      r"a_{b^c}",
                      r"{a_b}^c",
                      r"a^{b_c}",
                      r"{a^b}_c",
                      r"a_{b_c}",
                      r"{a_b}_c"]);
        should_fail!(errs,
                     parse,
                     [r"1_", r"1^", r"x_x_x", r"x^x_x^x", r"x^x^x", r"x_x^x_x"]);
        should_equate!(errs,
                       parse,
                       [(r"x_\alpha^\beta", r"x^\beta_\alpha"), (r"_2^3", r"^3_2")]);
        display_errors!(errs);
    }

    #[test]
    fn delimited() {
        let mut errs: Vec<String> = Vec::new();
        should_pass!(errs,
                     parse,
                     [r"\left(\right)",
                      r"\left.\right)",
                      r"\left(\right.",
                      r"\left\vert\right)",
                      r"\left(\right\vert"]);
        should_fail!(errs,
                     parse,
                     [r"\left1\right)",
                      r"\left.\right1",
                      r"\left",
                      r"\left.{1 \right."]);
        display_errors!(errs);
    }


    #[test]
    fn array() {
        const CORRECT_FORMULAS : &[&str] = &[
            r"\begin{array}{c}\end{array}",
            r"\begin{array}{c}1\\2\end{array}",
            r"\begin{array}{c}1\\\end{array}",
        ];

        for formula in CORRECT_FORMULAS.iter().cloned() {
            eprintln!("correct: {}", formula);
            parse(formula).unwrap();
        }

        const INCORRECT_FORMULAS : &[&str] = &[
            r"\begin{array}{c}",
            r"\begin{array}1\\2\end{array}",
        ];

        for formula in INCORRECT_FORMULAS.iter().cloned() {
            eprintln!("incorrect: {}", formula);
            assert!(parse(formula).is_err());
        }
    }

    #[test]
    fn test_custom_command() {
        let mut command_collection = CommandCollection::default();
        let command = CustomCommand::parse("#1 + #2").unwrap();
        command_collection.insert("add",  command).unwrap();
        let command = CustomCommand::parse(r"\lbrace#1\rbrace").unwrap();
        command_collection.insert("wrap", command).unwrap();

        let expected = parse("45 + 68");
        let got      = Parser::new(r"\add{45}{68}").with_custom_commands(&command_collection).parse();
        assert_eq!(expected, got);   

        // something before and after macros
        let expected = parse("145 + 681");
        let got      = Parser::new(r"1\add{45}{68}1").with_custom_commands(&command_collection).parse();
        assert_eq!(expected, got);   

        // commands in macro expansion
        let expected = parse(r"\frac{1}{2} + \frac{3}{4}");
        let got      = Parser::new(r"\add{\frac{1}{2}}{\frac{3}{4}}").with_custom_commands(&command_collection).parse();
        assert_eq!(expected, got);   

        // recursive macros
        let expected = parse("1 + 2 + 34");
        let got      = Parser::new(r"\add{1}{\add{2}{3}}4").with_custom_commands(&command_collection).parse();
        assert_eq!(expected, got);   

        // check that macro arg can't complete commands inside macro definition
        let expected = parse(r"\lbrace{}a\rbrace");
        let got      = Parser::new(r"\wrap{a}").with_custom_commands(&command_collection).parse();
        assert_eq!(expected, got);   

        // check that subsequent text can't complete commands inside macro def
        let expected = parse(r"\lbrace\rbrace{}a");
        let got      = Parser::new(r"\wrap{}a").with_custom_commands(&command_collection).parse();
        assert_eq!(expected, got);   
    }

    #[test]
    fn text_command() {
        let got = parse(r"\text{re + 43}").unwrap();
        let expected = vec![ParseNode::PlainText(PlainText {text : "re + 43".to_string()})];
        assert_eq!(expected, got);
    }


    #[test]
    fn test_parse_group() {
        fn generate_results(input : &str) -> (Option<ParseResult<Vec<ParseNode>>>, &str) {
            let mut parser = Parser::new(input);
            let result = parser.parse_group();
            (result, parser.input)
        }

        assert_eq!(generate_results("{}"), (Some(Ok(Vec::new())), ""));
    }

    #[test]
    fn test_end_of_parse() {
        let input = "1+1";
        let mut parser = Parser::new(input);
        assert!(!parser.end_of_parse().unwrap());

        let input = "";
        let mut parser = Parser::new(input);
        assert!(parser.end_of_parse().unwrap());

        let input = "} 1+1";
        let mut parser = Parser::new(input);
        parser.stop_parsing_at = Some(ParseDelimiter::CloseBracket);
        assert!(parser.end_of_parse().unwrap());

        let input = " } 1";
        let mut parser = Parser::new(input);
        parser.stop_parsing_at = Some(ParseDelimiter::CloseBracket);
        assert!(!parser.end_of_parse().unwrap());

        let input = "";
        let mut parser = Parser::new(input);
        parser.stop_parsing_at = Some(ParseDelimiter::CloseBracket);
        assert_eq!(parser.end_of_parse().unwrap_err(), ParseError::UnexpectedEof);
    }
}


