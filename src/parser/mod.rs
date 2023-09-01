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
use self::nodes::AtomChange;
use self::nodes::Color;
pub use self::nodes::ParseNode;
use self::nodes::PlainText;
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
    /// An alignment character `&`
    Alignment,
    /// End of line `\\`
    EndOfLine,
    /// An end of environment, e.g. `\end{array}`
    EndEnv(Environment),
    /// End of input
    Eof,
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
            ParseDelimiter::Alignment       => f.write_str(r"&"),
            ParseDelimiter::EndOfLine       => f.write_str(r"\\"),
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
        let final_delimiter = self.parse_expression()?;
        match final_delimiter {
            ParseDelimiter::Eof => Ok(self.result),
            delim => Err(ParseError::ExpectedDelimiter { found: delim, expected: ParseDelimiter::Eof })
        }
    }

    fn parse_expression(&mut self) -> ParseResult<ParseDelimiter> {
        loop {
            self.consume_whitespace();
            if let Some(parse_delimiter) = self.end_of_parse() {
                let parse_delimiter = parse_delimiter?;
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
    // TODO: could we pre-parse what's coming next to avoid reading multiple times what's coming up
    fn end_of_parse(&mut self) -> Option<ParseResult<ParseDelimiter>> {
        let input = self.input;
        if input.is_empty() {
            return Some(Ok(ParseDelimiter::Eof));
        }

        match self.control_sequence() {
            Some("middle") => return Some(Ok(ParseDelimiter::MiddleDelimiter)),
            Some("right")  => return Some(Ok(ParseDelimiter::RightDelimiter)),
            Some("end")    => return Some((|| {
                let env = self.parse_environment_name()?;
                Ok(ParseDelimiter::EndEnv(env))
            }) ()),
            Some(r"\")     => return Some(Ok(ParseDelimiter::EndOfLine)),
            Some(_) => {
                self.input = input; // rewind, the control sequence is useful
                return None;
            }
            _ => ()
        }

        if self.try_parse_char('}').is_some() {
            return Some(Ok(ParseDelimiter::CloseBracket));
        }

        if self.try_parse_char('&').is_some() {
            return Some(Ok(ParseDelimiter::Alignment));
        }

        None
    }


    /// Parses a sub- or a super-script
    #[inline]
    fn parse_script(&mut self, superscript : bool) -> Option<ParseResult<Vec<ParseNode>>> {
        let symbol = if superscript { '^' } else { '_' };
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
                    Command::Radical => self.parse_radical().map(ParseNode::Radical),
                    Command::Rule    => self.parse_rule().map(ParseNode::Rule),
                    Command::Color   => self.parse_color_command().map(ParseNode::Color),
                    Command::ColorLit(color) => self.parse_required_argument().map(|inner| ParseNode::Color(Color { color, inner, })),
                    Command::Fraction(left, right, thickness, math_style) => self.parse_fraction(left, right, thickness, math_style).map(|gen_frac| ParseNode::GenFraction(gen_frac)),
                    Command::DelimiterSize(_, _) => todo!(),
                    Command::Kerning(kern) => Ok(ParseNode::Kerning(kern)),
                    Command::Style(_) => todo!(),
                    Command::AtomChange(at) => self.parse_required_argument().map(|inner| ParseNode::AtomChange(AtomChange { at, inner, })),
                    Command::TextOperator(name, delim) => {
                        Ok(ParseNode::AtomChange(AtomChange {
                            at : AtomType::Operator(delim),
                            inner : vec![ParseNode::PlainText(PlainText { text: name.to_string() })],
                        }))
                    },
                    Command::SubStack(_) => todo!(),
                    Command::Text => todo!(),
                }
            }
            // second case: a symbol with a name
            else if let Some(mut symbol) = Symbol::from_name(control_seq_name) {
                // TODO: make this a method of symbol
                symbol.codepoint = style_symbol(symbol.codepoint, self.local_style);
                Ok(ParseNode::Symbol(symbol))
            }
            // third case: a delimiter
            else if control_seq_name == "left" {
                self.parse_delimited_sequence().map(|delim| ParseNode::Delimited(delim))
            } 
            // fourth case: an environment start, e.g. \begin{env}
            else if control_seq_name == "begin" {
                self.parse_env().map(ParseNode::Array)
            }
            // fifth case: a custom macro
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
            let codepoint = style_symbol(codepoint, self.local_style);
            Ok(Symbol { codepoint, atom_type, })
        })())
    }

    /// This method parses the two arguments that follow `\color`, namely a color name and a set of inner constituent
    fn parse_color_command(&mut self) -> ParseResult<Color> {
        let color_name = self.parse_group_as_string().ok_or(ParseError::RequiredMacroArg)?;
        let color = Parser::new(color_name).parse_color()?;
        let inner = self.parse_required_argument()?;
        Ok(Color { color, inner, })
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
    use crate::parser::{environments::Environment,};


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
            r"\begin{array}c1&3\\2&4\end{array}",
        ];

        for formula in CORRECT_FORMULAS.iter().cloned() {
            eprintln!("correct: {}", formula);
            parse(formula).unwrap();
        }

        const INCORRECT_FORMULAS : &[&str] = &[
            r"\begin{array}{c}",
            r"\begin{array}1\\2\end{array}",
            r"\begin{array}c}1&3\\2&4\end{array}",
        ];

        for formula in INCORRECT_FORMULAS.iter().cloned() {
            eprintln!("incorrect: {}", formula);
            parse(formula).unwrap_err();
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
        assert_eq!(parser.end_of_parse(), Some(Ok(ParseDelimiter::Eof)));

        let input = "} 1+1";
        let mut parser = Parser::new(input);
        assert_eq!(parser.end_of_parse(), Some(Ok(ParseDelimiter::CloseBracket)));

        let input = r"\end{bmatrix} 1+1";
        let mut parser = Parser::new(input);
        assert_eq!(parser.end_of_parse(), Some(Ok(ParseDelimiter::EndEnv(Environment::BMatrix))));

        let input = r"& \end{bmatrix}";
        let mut parser = Parser::new(input);
        assert_eq!(parser.end_of_parse(), Some(Ok(ParseDelimiter::Alignment)));

        let input = r"\\";
        let mut parser = Parser::new(input);
        assert_eq!(parser.end_of_parse(), Some(Ok(ParseDelimiter::EndOfLine)));

        let input = " } 1";
        let mut parser = Parser::new(input);
        assert_eq!(parser.end_of_parse(), None);

    }

    #[test]
    fn test_escape_character() {
        todo!()
    }


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

}
