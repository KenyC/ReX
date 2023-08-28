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
pub mod utils;
pub mod environments;
pub mod functions;
pub mod lexer;
pub mod error;

use font::expect;

use self::lexer::expect_left;
use self::lexer::expect_middle;
use self::lexer::expect_right;
pub use self::nodes::ParseNode;
pub use self::nodes::is_symbol;




use std::fmt::Display;
use std::todo;
use std::unreachable;

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
use crate::parser::utils::fmap;
use crate::dimensions::AnyUnit;


/// Any sequence of characters that marks the end of something, a command (e.g. closing bracket), an environment (e.g. `\end{array}`) or delimiters
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ParseDelimiter {
    /// A closing bracket `}`
    CloseBracket,
    /// A middle delimiter `\middle`
    MiddleDelimiter,
    /// A middle delimiter `\right`
    RightDelimiter,
    /// An end of environment, e.g. `\end{array}`
    EndEnv(Environment),
    /// End of input
    Eof
}

impl ParseDelimiter {
    fn expect(&self, expected: ParseDelimiter) -> ParseResult<()> {
        if *self == expected {
            Ok(())
        }
        else {
            Err(ParseError::ExpectedDelimiter { found: *self, expected })
        }
    }

}

impl Display for ParseDelimiter {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ParseDelimiter::CloseBracket    => f.write_str("}"),
            ParseDelimiter::MiddleDelimiter => f.write_str(r"\middle"),
            ParseDelimiter::RightDelimiter  => f.write_str(r"\right"),
            ParseDelimiter::EndEnv(name)    => write!(f, r"\end{{{}}}", name),
            ParseDelimiter::Eof             => f.write_str(r"end of input"),
        }
    }
}

/// A parser, contains some input, some parameters of parsing such as custom commands and some parser state info.  
/// The lifetime `'i` is for the borrow of the input.
/// The lifetime `'c` is for the borrow of the custom command collection (cf [`CommandCollection`]).
pub struct Parser<'i, 'c> {
    input : & 'i str,
    local_style : Style,
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

    fn parse_expression(&mut self) -> ParseResult<ParseDelimiter> {
        loop {
            self.consume_whitespace();
            if let Some(parse_delimiter) = self.end_of_parse() {
                return Ok(parse_delimiter);
            }


            // We try to parse the first things that comes along
            let node = 
                self.parse_control_sequence()
                .or_else(|| fmap(self.parse_group(),  |nodes|  ParseNode::Group(nodes)))
                .or_else(|| fmap(self.parse_symbol(), |symbol| ParseNode::Symbol(symbol)))
            ;
            let node = node.ok_or_else(|| todo!())??;

            // Take the result of `parse_script` and shift it from `Option<Result<..>>` to `Result<Option<..>>`
            let mut subscript   = self.parse_script(false).map_or(Ok(None), |maybe_arg| maybe_arg.map(Some))?;
            let mut superscript = self.parse_script(true).map_or(Ok(None),  |maybe_arg| maybe_arg.map(Some))?;
            // scripts may be in any order ; so we try to parse a subscript again (just in case superscript came first)
            // TODO : excessive subscript error
            if subscript.is_none() {
                subscript   = self.parse_script(false).map_or(Ok(None), |maybe_arg| maybe_arg.map(Some))?;
            }

            let node =
                if subscript.is_some() || superscript.is_some() {
                    ParseNode::Scripts(Scripts { 
                        base: Some(Box::new(node)), 
                        superscript, subscript,
                    })
                }
                else 
                { node };

            // We add the result to our nodes
            self.result.push(node);
        }
    }

    /// Recovers the nodes that have been parsed yet, without running any more parsing
    pub fn to_results(self) -> Vec<ParseNode> {
        self.result
    }


    /// Check if parser has reached end of parse.   
    /// This maybe because we've hit end of input or if `Parser::stop_parsing_at` is set, because we've reached a delimiter. 
    /// Note that when the condition for end of parse is '}' or another delimiter, `self.input` is not ad
    fn end_of_parse(&mut self) -> Option<ParseDelimiter> {
        let input = self.input;
        if input.is_empty() {
            return Some(ParseDelimiter::Eof);
        }

        match self.control_sequence() {
            Some("middle") => return Some(ParseDelimiter::MiddleDelimiter),
            Some("right")  => return Some(ParseDelimiter::RightDelimiter),
            Some("end")    => todo!(),
            Some(_) => {
                self.input = input; // rewind, the control sequence is useful
                return None;
            }
            _ => ()
        }

        if self.try_parse_char('}').is_some() {
            return Some(ParseDelimiter::CloseBracket);
        }

        None
    }


    /// Parses a sub- or a super-script
    #[inline]
    fn parse_script(&mut self, superscript : bool) -> Option<ParseResult<Vec<ParseNode>>> {
        let symbol = if superscript { '_' } else { '^' };
        self.try_parse_char(symbol)?;
        Some(self.parse_required_argument())
    }

    /// Expects to parse something of the form `\foo[..]{..}` with correct number of arguments and well-typed arguments
    /// If unable to, it does not advance input.
    fn parse_control_sequence(&mut self) -> Option<ParseResult<ParseNode>> {
        let custom_commands = self.custom_commands;
        let control_seq_name = self.control_sequence()?;

            
        // When the command name has been recognized, we are sure this is a command
        // Any failure from now on must be a parsing error (misformed input)
        Some(
            // TODO: think of best ordering of these cases
            // First case, a TeX command
            if let Some(command) = Command::from_name(control_seq_name) {
                match command {
                    Command::Radical => self.parse_radical().map(|radical| ParseNode::Radical(radical)),
                    Command::Rule => todo!(),
                    Command::Color => todo!(),
                    Command::ColorLit(_) => todo!(),
                    Command::Fraction(left, right, thickness, math_style) => self.parse_fraction(left, right, thickness, math_style).map(|gen_frac| ParseNode::GenFraction(gen_frac)),
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
            // third case: a delimiter
            else if control_seq_name == "left" {
                self.parse_delimited_sequence().map(|delim| ParseNode::Delimited(delim))
            } 
            // a \middle or \right delimiter may be caught here
            // this happens for one of two reasons:
            // - a \right without a corresponding \left
            // - there is a corresponding \left but it is separated from \right a bracket or anything
            // either way, it's an ill-formed input
            else if control_seq_name == "middle" {
                Err(ParseError::UnexpectedMiddle)
            }
            else if control_seq_name == "right" {
                Err(ParseError::UnexpectedRight)
            }
            // fourth case: a custom macro
            else if let Some(command) = custom_commands.and_then(|collection| collection.query(control_seq_name)) {
                todo!()
            }
            else {
                Err(todo!())
            }
        )

    }

    /// Assuming `\left` has just been parsed, parses a `\left<char> .. (\middle<char>)* .. \right<char>` sequence.
    fn parse_delimited_sequence(&mut self) -> ParseResult<Delimited> {

        let mut hasnt_reached_right_delimiter = true;
        let mut delimiters = Vec::with_capacity(2);
        let mut inners     = Vec::with_capacity(1);

        let left_delimiter = self.parse_delimiter().ok_or(ParseError::MissingSymbolAfterDelimiter)??;
        let left_delimiter = expect_left(left_delimiter)?;

        delimiters.push(left_delimiter);
        while hasnt_reached_right_delimiter {
            let mut parser = self.fork();
            let stopped_at = parser.parse_expression()?;
            self.input = parser.input;
            let inner = parser.result;
            inners.push(inner);

            hasnt_reached_right_delimiter = match stopped_at {
                ParseDelimiter::MiddleDelimiter => true,
                ParseDelimiter::RightDelimiter  => false,
                // TODO: report a more correct error that mentions `\middle` as another token to be expected
                other => return Err(ParseError::ExpectedDelimiter { found: other, expected: ParseDelimiter::RightDelimiter }),
            };

            let delimiter = self.parse_delimiter().ok_or_else(|| todo!())??;
            // if delimiter isn't of the right type we trigger an error
            let delimiter = if hasnt_reached_right_delimiter {
                expect_middle(delimiter)?
            }
            else {
                expect_right(delimiter)?
            };

            delimiters.push(delimiter);
        }
        Ok(Delimited::new(delimiters, inners,))
    }

    /// Parses a symbol
    fn parse_symbol(&mut self) -> Option<ParseResult<Symbol>> {
        let codepoint = self.parse_char()?;
        // This baroque closure construction allows us to use `?` to propagate an error to the ParseResult
        // Otherwise, the `?` would propagate to `Option`.
        Some((|| {
            let atom_type = codepoint_atom_type(codepoint).ok_or_else(|| ParseError::UnrecognizedSymbol(codepoint))?;
            Ok(Symbol { codepoint, atom_type, })
        })())
    }

    /// Creates a new parser in the same state, with no nodes
    fn fork(&self) -> Self {
    	let Self { input, local_style, custom_commands, .. } = self;
    	Self { 
    		input, 
    		local_style : local_style.clone(), 
    		custom_commands : *custom_commands, 
    		result: Vec::new(),
    	}
    }

    fn parse_group(&mut self) -> Option<ParseResult<Vec<ParseNode>>> {
        let mut parser = self.fork();

        parser.try_parse_char('{')?;

        // parser.stopped_parsing_at = Some(ParseDelimiter::CloseBracket);
        Some((|| {
            let result = parser.parse_expression()?;
            self.input = parser.input;
            result.expect(ParseDelimiter::CloseBracket)?;


            Ok(parser.to_results())

        }) ())
    }

    fn parse_delimiter(&mut self) -> Option<ParseResult<Symbol>> {
        self.control_sequence().and_then(Symbol::from_name).map(Ok) // either a control sequence named symbol
            .or_else(|| self.parse_symbol()) // or a genuine symbol
    }
}


/// This function is the API entry point for parsing tex.
pub fn parse(input: &str) -> ParseResult<Vec<ParseNode>> {
    Parser::new(input).parse()
}


/// Utility function for packaging multiply wrapped node
fn group(nodes: Option<Result<Vec<ParseNode>, ParseError>>) -> Option<Result<ParseNode, ParseError>> {
    match nodes {
        Some(Ok(node)) => Some(Ok(ParseNode::Group(node))),
        Some(Err(e))   => Some(Err(e)),
        None           => None,
    }
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
        parse("h").unwrap();
    }

    #[test]
    fn ldots() {
        let mut errs: Vec<String> = Vec::new();
        should_pass!(errs, parse, [r"\ldots",r"\vdots",r"\dots"]);
        display_errors!(errs);
    }

    #[test]
    fn fractions() {
        let success_cases = vec![r"\frac\alpha\beta", r"\frac\int2"];
        for case in success_cases {
            eprintln!("{} should pass", case);
            let result = parse(case);
            result.unwrap();
        }

        let failure_cases = vec![r"\frac \left(1 + 2\right) 3"];
        for case in failure_cases {
            eprintln!("{} should fail", case);
            let result = parse(case);
            assert!(result.is_err());
        }

        let equality_cases = vec![
            (r"\frac12", r"\frac{1}{2}"),
            (r"\frac \sqrt2 3", r"\frac{\sqrt2}{3}"),
            (r"\frac \frac 1 2 3", r"\frac{\frac12}{3}"),
            (r"\frac 1 \sqrt2", r"\frac{1}{\sqrt2}")
        ];
        for (case1, case2) in equality_cases {
            eprintln!("{} == {}", case1, case2);
            assert_eq!(parse(case1), parse(case2));
        }
    }

    #[test]
    fn radicals() {
        let mut errs: Vec<String> = Vec::new();
        // TODO: Add optional paramaters for radicals
        let success_cases = vec![
            r"\sqrt{x}",
            r"\sqrt2",
            r"\sqrt\alpha",
            r"1^\sqrt2",
            r"\alpha_\sqrt{1+2}",
            r"\sqrt\sqrt2"
        ];
        for case in success_cases {
            eprintln!("{}", case);
            let result = parse(case);
            result.unwrap();
        }

        let failure_cases = vec![r"\sqrt", r"\sqrt_2", r"\sqrt^2"];
        for case in failure_cases {
            eprintln!("{}", case);
            let result = parse(case);
            assert!(result.is_err());
        }

        let equality_cases = vec![
            (r"\sqrt2", r"\sqrt{2}"),
        ];
        for (case1, case2) in equality_cases {
            eprintln!("{} == {}", case1, case2);
            assert_eq!(parse(case1), parse(case2));
        }

        let inequality_cases = vec![
            (r"\sqrt2_3", r"\sqrt{2_3}"),
        ];
        for (case1, case2) in inequality_cases {
            eprintln!("{} != {}", case1, case2);
            assert!(parse(case1) != parse(case2));
        }
    }

    #[test]
    fn scripts() {
        let success_cases = vec![
            r"1_2^3",
            // r"_1", // TODO : decide whether initial sub and superscript are allowedd
            // r"^\alpha",
            // r"_2^\alpha",
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
            r"{a_b}_c"
        ];
        for case in success_cases {
            eprintln!("{}", case);
            let result = parse(case);
            result.unwrap();
        }

        let failure_cases = vec![r"1_", r"1^", r"x_x_x", r"x^x_x^x", r"x^x^x", r"x_x^x_x"];
        for case in failure_cases {
            eprintln!("{}", case);
            let result = parse(case);
            assert!(result.is_err());
        }


        let equality_cases = vec![
            (r"x_\alpha^\beta", r"x^\beta_\alpha"), 
            // (r"_2^3", r"^3_2"),
        ];
        for (case1, case2) in equality_cases {
            eprintln!("{} == {}", case1, case2);
            assert_eq!(parse(case1), parse(case2));
        }
    }

    #[test]
    fn delimited() {
        let success_cases = vec![
            r"\left(\right)",
            r"\left.\right)",
            r"\left(\right.",
            r"\left\vert\right)",
            r"\left(\right\vert",
            r"\left(\middle.\right\vert",
        ];
        for case in success_cases {
            eprintln!("{}", case);
            let result = parse(case);
            result.unwrap();
        }

        let failure_cases = vec![
            r"\left1\right)",
            r"\left.\right1",
            r"\left",
            r"\left.{1 \right.",
            r"\left(\middle(\right\vert",
        ];
        for case in failure_cases {
            eprintln!("{}", case);
            let result = parse(case);
            assert!(result.is_err());
        }
    }


    #[test]
    fn array() {
        const CORRECT_FORMULAS : &[&str] = &[
            r"\begin{array}{c}\end{array}",
            r"\begin{array}{c}1\\2\end{array}",
            r"\begin{array}{c}1\\\end{array}",
            r"\begin{array}{cl}1&3\\2&4\end{array}",
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
        assert_eq!(parser.end_of_parse(), None);

        let input = "";
        let mut parser = Parser::new(input);
        assert_eq!(parser.end_of_parse(), Some(ParseDelimiter::Eof));

        let input = "} 1+1";
        let mut parser = Parser::new(input);
        assert_eq!(parser.end_of_parse(), Some(ParseDelimiter::CloseBracket));

        let input = " } 1";
        let mut parser = Parser::new(input);
        assert_eq!(parser.end_of_parse(), None);

    }

    #[test]
    fn test_escape_character() {
        todo!()
    }
}


