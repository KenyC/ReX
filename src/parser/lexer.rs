//! Producing tokens for the parser
use std::{fmt, todo};
use crate::dimensions::{AnyUnit, Unit};
use super::color::RGBA;
use crate::parser::error::{ParseError, ParseResult};


/// A token for LateX
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Token<'a> {
    /// A TeX command or macro, e.g. `\begin` or `\sqrt`
    Command(&'a str),
    /// A series of whitespaces
    WhiteSpace,
    /// A symbol, anything not covered by the above
    Symbol(char),
    /// End of file token
    EOF,
}


impl<'a> fmt::Display for Token<'a> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            Token::Command(cmd) => write!(f, r#""\{}""#, cmd),
            Token::Symbol(c) => write!(f, r"'{}'", c),
            Token::WhiteSpace => write!(f, r"' '"),
            Token::EOF => write!(f, "EOF"),
        }
    }
}

/// The main structure for producing tokens from an input string
#[derive(Clone, Debug)]
pub struct Lexer<'a> {
    input : & 'a str,
}

impl<'a> Lexer<'a> {
    /// Create a new lexer, whose current token is the first token
    /// to be processed.
    pub fn new(input: &'a str) -> Lexer<'a> {
        Self { input }
    }

    /// Advances through the input so that the first character pointed to
    /// is not a whitespace
    pub fn consume_whitespace(&mut self) {
        let mut chars = self.input.chars();

        while chars.next().map_or(false, |c| c.is_whitespace()) 
        {}

        self.input = chars.as_str();
    }


    /// Attempts parsing a control sequence like `\bla`, returning `bla`.
    pub fn control_sequence(&mut self) -> Option<& 'a str> {
        let mut chars = self.input.chars();
        if chars.next() != Some('\\') {
            return None;
        }

        let start_command = chars.as_str();

        // TODO: problem with next condition
        // A \ at the end of input is considered the same as a slash followed by a space
        let character = chars.next().unwrap_or(' ');


        // If the first character is non-alphabetic, that is the command and we return it
        if !character.is_ascii_alphabetic() {
            let suffix = chars.as_str();
            self.input = suffix;
            return Some(diff_slices(start_command, suffix));
        }

        // Otherwise, we keep looping while characters are ASCII alphabetic
        let mut end_command = chars.as_str();
        while chars.next().map_or(false, |c| c.is_ascii_alphabetic()) {
            end_command = chars.as_str();
        }

        self.input = end_command;

        return Some(diff_slices(start_command, end_command));
    }

    /// Returns wrapped input
    pub fn input(&self) -> & 'a str {
        self.input 
    }
}

/// Assuming `slice2` is a suffix of `slice1`, 
/// returns the prefix of `slice1` that ends just before the first character of `slice2`
fn diff_slices<'a>(slice : & 'a str, suffix : & 'a str) -> & 'a str {
    &slice[.. (slice.len() - suffix.len())]
}


#[cfg(test)]
mod tests {
    use std::todo;

    use rand::Rng;

    use crate::dimensions::AnyUnit;

    use super::{Lexer, Token};



    #[test]
    fn lex_primes() {
        let mut lexer  = Lexer::new("a'b''c'''d");
        let mut tokens = Vec::new();

        loop {
            let token : Token = todo!();
            // let token = lexer.current();
            if token == Token::EOF {break;}
            tokens.push(token);
            todo!();
            // lexer.next();
        }

        let expected = [
            Token::Symbol('a'), 
            Token::Command("prime"),
            Token::Symbol('b'), 
            Token::Command("dprime"),
            Token::Symbol('c'), 
            Token::Command("trprime"),
            Token::Symbol('d'), 
        ];
        assert_eq!(
            tokens,
            expected.to_vec(),
        )
    }

    #[test]
    fn lex_control_sequence() {
        let tests = [
            (r"\cal 0",     Some("cal"), " 0"),
            (r"\$ 0",       Some("$"),   " 0"),
            (r"\cal{} 0",   Some("cal"), "{} 0"),
            (r"\c{} 0",     Some("c"),   "{} 0"),
            (r"\",          Some(""),    ""),
            (r"\ +1",       Some(" "),   "+1"),
            (r"_1",         None,        "_1"),
        ];

        for (input, name, remainder) in tests {
            eprintln!("Input: {:?}", input);
            let mut lexer : Lexer = Lexer::new(input);
            let control_sequence = lexer.control_sequence();
            assert_eq!(control_sequence, name);
            assert_eq!(lexer.input(), remainder);
        }
    }


    #[test]
    fn lex_tokens() {
        macro_rules! assert_eq_token_stream {
            ($left:expr, $right:expr) => {{
                let mut left  = Lexer::new($left);
                let mut right = Lexer::new($right);

                loop {
                    let l_tok : Token = todo!();
                    let r_tok : Token = todo!();
                    // let l_tok = left.next();
                    // let r_tok = right.next();

                    assert_eq!(l_tok, r_tok);
                    if l_tok == Token::EOF {
                        break
                    }
                }
            }}
        }

        assert_eq_token_stream!(r"\cs1", r"\cs 1");
        assert_eq_token_stream!(r"\cs1", r"\cs    1");
        assert_eq_token_stream!(r"\cs?", "\\cs\n\n\t?");
        assert_eq_token_stream!(r"\test\test", r"\test   \test");
        assert_eq_token_stream!(r"1     +       2", r"1 + 2");
        assert_eq_token_stream!(r"123\", "123");
    }

    #[test]
    fn lex_group() {
        todo!()

        // let mut l = Lexer::new("{1}");
        // assert_eq!(l.group(), Ok("1"));
        // assert!(!(l.current() == Token::EOF));

        // let mut l = Lexer::new("   {  abc } ");
        // assert_eq!(l.group(), Ok("  abc "));
        // assert!(!(l.current() == Token::EOF));

        // let mut l = Lexer::new("{}");
        // assert_eq!(l.group(), Ok(""));
        // assert!(!(l.current() == Token::EOF));


        // let mut l = Lexer::new("{fez{fe}}");
        // assert_eq!(l.group(), Ok("fez{fe}"));
        // assert!(!(l.current() == Token::EOF));

        // let mut l = Lexer::new(r"{fez\{}");
        // assert_eq!(l.group(), Ok(r"fez\{"));
        // assert!(!(l.current() == Token::EOF));


        // // This doesn't seem correct:
        // // assert_group!("{{}}", Ok("{"));
    }

    #[test]
    fn lex_alphanumeric() {
        macro_rules! assert_alphanumeric {
            ($input:expr, $result:expr) => {
                let mut lex = Lexer::new($input);
                todo!()
                // assert_eq!(lex.alphanumeric(), $result);
            }
        }

        // Ends on EOF
        assert_alphanumeric!("abc", "abc");
        assert_alphanumeric!("", "");

        // Ends on Whitespace
        assert_alphanumeric!("123 ", "123");
        assert_alphanumeric!(" 123", "");

        // End on Command
        assert_alphanumeric!(r"\pi2", "");
        assert_alphanumeric!(r"2\alpha", "2");

        // End on non-alphanumeric
        assert_alphanumeric!("{abc}", "");
        assert_alphanumeric!("abc!", "abc");
    }

    #[test]
    fn lex_dimension() {
        fn parse_dim(input : &str) -> Option<AnyUnit> {
            let mut lexer = Lexer::new(input);
            todo!();
            // lexer.dimension()  
        }


        assert_eq!(parse_dim(r"123px abc"),    Some(AnyUnit::Px(123.0)));
        assert_eq!(parse_dim(r"1.23em abc"),   Some(AnyUnit::Em(1.23)));
        assert_eq!(parse_dim(r"- 1.23em 123"), Some(AnyUnit::Em(-1.23)));
        assert_eq!(parse_dim(r"+1.34px 134"),  Some(AnyUnit::Px(1.34)));
        assert_eq!(parse_dim("-   12em"),      Some(AnyUnit::Em(-12.0)));
        assert_eq!(parse_dim("+   12px"),      Some(AnyUnit::Px(12.0)));
        assert_eq!(parse_dim("-  .12em"),      Some(AnyUnit::Em(-0.12)));
        assert_eq!(parse_dim("00.123000em"),   Some(AnyUnit::Em(0.123)));
        assert_eq!(parse_dim("001.10000em"),   Some(AnyUnit::Em(1.1)));

        assert_eq!(parse_dim(r"px"),       None);
        assert_eq!(parse_dim(r"..em"),     None);
        assert_eq!(parse_dim(r"1.4.1em"),  None);

    }
}
