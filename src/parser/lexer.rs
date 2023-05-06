use std::fmt;
use crate::dimensions::Unit;
use super::color::RGBA;
use crate::error::{ParseError, ParseResult};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Token<'a> {
    Command(&'a str),
    Symbol(char),
    WhiteSpace,
    EOF,
}

impl<'a> Token<'a> {
    #[cfg_attr(rustfmt, rustfmt_skip)]
    pub fn ends_expression(self) -> bool {
        match self {
            Token::EOF
            | Token::Symbol('}')
            | Token::Command("right")  // middle ends the group that started with a previous 'left' or 'middle'
            | Token::Command("middle")
            | Token::Command(r"\")
            | Token::Command(r"end")
            | Token::Command(r"cr") => true,
            _ => false,
        }
    }

    pub fn expect(&self, expected: Token<'a>) -> ParseResult<'a, ()> {
        if *self == expected {
            Ok(())
        } else {
            Err(ParseError::ExpectedTokenFound(expected, (*self).into()))
        }
    }

    pub fn expect_command(self, expected: &'static str) -> ParseResult<'a, ()> {
        self.expect(Token::Command(expected))
    }

    pub fn expect_symbol(self, expected: char) -> ParseResult<'a, ()> {
        self.expect(Token::Symbol(expected))
    }

    pub fn expect_whitespace(self) -> ParseResult<'a, ()> {
        self.expect(Token::WhiteSpace)
    }

    pub fn expect_eof(self) -> ParseResult<'a, ()> {
        self.expect(Token::EOF)
    }
}

#[derive(Clone, Debug)]
pub struct Lexer<'a> {
    /// The current input string being lexed.
    ///
    /// The [`Lexer::current`] token was lexed from that part of the string before.
    ///
    /// EMPTYNESS GUARANTEE: if [`Lexer::input`] is empty, so is [`Lexer::next_inputs`].
    /// This way, [`Lexer::input`] is not empty unless we're at the end of the string.
    input: &'a str,

    /// .The next input strings to be lexed after [`Lexer::input`]
    ///
    /// When [`Lexer::input`] becomes empty, the last extry of [`Lexer::next_inputs`] is popped and become [`Lexer::input`].
    next_inputs : Vec<&'a str>,

    /// The last token which has been lexed.
    current: Token<'a>,
}

impl<'a> Lexer<'a> {
    /// Create a new lexer, whose current token is the first token
    /// to be processed.
    pub fn new(input: &'a str) -> Lexer<'a> {
        let mut lex = Lexer {
            input: input,
            next_inputs: Vec::new(),
            current: Token::EOF,
        };

        lex.next();
        lex
    }

    /// Advanced to the next token to be processed, and return it.
    /// This will also modify `Lexer.current`.
    pub fn next(&mut self) -> Token<'a> {
        self.current = match self.next_char() {
            Some(c) if c.is_whitespace() => {
                self.advance_while_whitespace();
                Token::WhiteSpace
            }
            Some('\\') => self.control_sequence(),
            Some('\'') => self.sequence_of_primes(), // just like LateX, we replace apostrophes with primes
            Some(c) => Token::Symbol(c),
            None => Token::EOF,
        };

        debug!("{:?}", self.current);
        self.current
    }

    /// If the current token being processed from the lexer
    /// is a `WhiteSpace` then continue to call `.next()`
    /// until `lex.current` is the first non-whitespace token.
    /// This method is indepotent, so that calling this method
    /// twice has no effect.
    pub fn consume_whitespace(&mut self) {
        if self.current != Token::WhiteSpace {
            return;
        }
        self.advance_while_whitespace();
        self.next();
    }

    /// This method is the same as [consume_whitespace],
    /// except that it does not process the next token.
    fn advance_while_whitespace(&mut self) {
        while let Some(c) = self.current_char() {
            if !c.is_whitespace() {
                break;
            }
            self.advance_by(c.len_utf8());
        }
    }

    fn advance_by(&mut self, size : usize) {
        let len = self.input.len();
        if len <= size {
            self.input = &self.input[0 .. 0];
            if self.pop_input() {
                self.advance_by(size - len);
            }
        }
        else {
            self.input = &self.input[size ..];
        }
    }


    fn pop_input(&mut self) -> bool {
        if let Some(input) = self.next_inputs.pop() {
            self.input = input;
            true
        }
        else {
            false
        }
    }


    /// Lex a control sequence.  This method assumes that
    /// `self.pos` points to the first character after `\`.
    /// The cursor will advance through the control sequence
    /// name, and consume all whitespace proceeding.  When
    /// complete `self.current`'s  will start with the first character
    /// of the next item to be lexed. This function does not parse a command name
    /// across input boudaries ; the command name must wholly reside in [`Lexer::input`]
    /// (It is implicitly assumed that [`Lexer::input`] is separated from [`Lexer::next_input`] by empty groups)
    fn control_sequence(&mut self) -> Token<'a> {
        let start = self.input;

        // The first character is special in that a non-alphabetic
        // character is valid, but will terminate the lex.
        let total_advance = match self.current_char() {
            None => return Token::EOF,
            Some(c) if !c.is_alphabetic() => c.len_utf8(),
            _ => {
                // Otherwise Proceed until the first non alphabetic.
                start
                    .char_indices()
                    .take_while(|(_, c)| c.is_alphabetic())
                    .map(|(i, c)| i + c.len_utf8())
                    .last()
                    .unwrap_or(0)
            }
        };
        self.advance_by(total_advance);


        // Consume all whitespace proceeding a control sequence
        self.advance_while_whitespace();
        Token::Command(&start[ .. total_advance])
    }

    /// This method will parse a dimension.  It assumes
    /// that the lexer is currently pointed to the first valid
    /// character in a dimension.  So it may be necessary to
    /// consume_whitespace() prior to using this method.
    pub fn dimension(&mut self) -> ParseResult<'a, Option<Unit>> {
        // utter crap, rewrite.
        unimplemented!()
    }

    // TODO: may be needed to parse \newcommand
    /// Expect to find an {<inner>}, and return <inner>
    // pub fn group(&mut self) -> ParseResult<'a, &'a str> {
    //     self.consume_whitespace();
    //     self.current.expect(Token::Symbol('{'))?;

    //     let i_closing = match self.input.find('}') {
    //         Some(pos) => pos,
    //         None => return Err(ParseError::NoClosingBracket),
    //     };

    //     // Place cursor immediately after }
    //     let group_inside = &self.input[.. i_closing];
    //     self.advance_by(i_closing + 1);
    //     self.next();
    //     Ok(group_inside)
    // }

    /// Match a segment of alphanumeric characters.  This method will
    /// return an empty string if there are no alphanumeric characters.
    pub fn alphanumeric(&mut self) -> String {
        let mut to_return = String::new();


        // This method expects that the next "Token" is a sequence of
        // alphanumerics.  Since `current_char` points at the first
        // non-parsed token, we must check the current Token to proceed.
        match self.current {
            Token::Symbol(c) if c.is_alphanumeric() => to_return.push(c),
            _ => return String::new(),
        };

        let start = self.input;
        while let Some(c) = self.current_char() {
            if !c.is_alphanumeric() {
                break;
            }
            self.advance_by(c.len_utf8());
        }
        let total_advance = start.len() - self.input.len();
        to_return.push_str(&start[.. total_advance]);
        self.next();
        to_return
    }

    // Match a valid Color.  A color is defined as either:
    //   1. Alphabetic name for a valid CSS color.
    //   2. #RRGGBB (that is a # followed by 6 digits)
    //   3. #RRGGBBAA (that is a # followed by 8 digits)

    pub fn color(&mut self) -> ParseResult<'a, RGBA> {
        unimplemented!()
    }

    fn next_char(&mut self) -> Option<char> {
        match self.current_char() {
            None => None,
            Some(c) => {
                self.advance_by(c.len_utf8());
                Some(c)
            }
        }
    }

    fn current_char(&mut self) -> Option<char> {
        self.input.chars().next()
    }


    fn sequence_of_primes(&mut self) -> Token<'a> {
        const PRIME_LEN : usize = '\''.len_utf8();

        if !matches!(self.current_char(), Some('\'')) {
            return Token::Command("prime");
        }
        self.advance_by(PRIME_LEN);

        if !matches!(self.current_char(), Some('\'')) {
            return Token::Command("dprime");
        }
        self.advance_by(PRIME_LEN);

        Token::Command("trprime")
    }

    /// The token currently being processed.
    #[inline]
    pub fn current(&self) -> Token<'a> {
        self.current
    }
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

#[cfg(test)]
mod tests {
    use rand::Rng;

    use super::{Lexer, Token};

    impl<'a> Lexer<'a> {
        // If `Lexer::input` is empty, so must `Lexer::next_input` be
        fn check_input_not_empty_guarantee(&self) -> bool {
            return !self.input.is_empty() || self.next_inputs.is_empty();
        }
    }

    #[test]
    fn advance_does_not_break_input_not_empty_guarantee() {
        let mut rng = rand::thread_rng();

        fn generate_random_string(length: usize) -> String {
            let mut rng = rand::thread_rng();
            let chars: String = (0..length).map(|_| {
                let random_char : u32 = rng.gen_range(('a' as u32) .. ('z' as u32));
                char::from_u32(random_char).unwrap()
            }).collect();

            chars
        }


        for _ in 0 .. 50 {
            let input  = generate_random_string(rng.gen_range(1 .. 5));
            let mut next_inputs_owned = Vec::new();
            for _ in 0 .. 10 {
                next_inputs_owned.push(generate_random_string(rng.gen_range(0 .. 5)));
            }

            let next_inputs : Vec<&str> = next_inputs_owned.iter().map(|x| x.as_ref()).collect();

            let mut lexer = Lexer {
                input : &input,
                next_inputs: next_inputs,
                current: Token::EOF,
            };

            for _ in 0 .. 20 {
                // All strings are ASCII so no byte advance will create a bug
                let advance = rng.gen_range(0 .. 5);
                lexer.advance_by(advance);
                assert!(lexer.check_input_not_empty_guarantee());
            }
        }

    }

    #[test]
    fn lex_primes() {
        let mut lexer  = Lexer::new("a'b''c'''d");
        let mut tokens = Vec::new();

        loop {
            let token = lexer.current;
            if token == Token::EOF {break;}
            tokens.push(token);
            lexer.next();
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
            (r"cal 0",     Token::Command("cal"), "0"),
            (r"$ 0",       Token::Command("$"),   "0"),
            (r"cal{} 0",   Token::Command("cal"), "{} 0"),
            (r"c{} 0",     Token::Command("c"),   "{} 0"),
        ];

        for (input, token, remainder) in tests {
            let mut lexer = Lexer {
                input,
                next_inputs: Vec::new(),
                current: Token::EOF,
            };
            assert_eq!(lexer.control_sequence(), token);
            assert_eq!(lexer.input, remainder);
        }
    }


    #[test]
    fn lex_tokens() {
        macro_rules! assert_eq_token_stream {
            ($left:expr, $right:expr) => {{
                let mut left  = Lexer::new($left);
                let mut right = Lexer::new($right);

                loop {
                    let l_tok = left.next();
                    let r_tok = right.next();

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

    // TODO: may be needed to parse \newcommand
    // #[test]
    // fn lex_group() {
    //     macro_rules! assert_group {
    //         ($input:expr, $result:expr) => {
    //             let mut l = Lexer::new($input);
    //             assert_eq!(l.group(), $result);
    //             assert!(!(l.current == Token::Symbol('}')));
    //         }
    //     }

    //     assert_group!("{1}", Ok("1"));
    //     assert_group!("   {  abc } ", Ok("  abc "));
    //     assert_group!("{}", Ok(""));

    //     // This doesn't seem correct:
    //     // assert_group!("{{}}", Ok("{"));
    // }

    #[test]
    fn lex_alphanumeric() {
        macro_rules! assert_alphanumeric {
            ($input:expr, $result:expr) => {
                let mut lex = Lexer::new($input);
                assert_eq!(lex.alphanumeric(), $result);
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

    // #[test]
    // fn lex_dimension() {
    //     use dimensions::Unit;
    //     macro_rules! assert_dim {
    //         ($input:expr, $result:expr) => (
    //             let mut _l = Lexer::new($input);
    //             assert_eq!(_l.dimension().unwrap(), Some(Unit::Px($result)));
    //         )
    //     }

    //     assert_dim!(r"123 abc", 123.0);
    //     assert_dim!(r"1.23 abc", 1.23);
    //     assert_dim!(r"- 1.23 123", -1.23);
    //     assert_dim!(r"+1.34 134", 1.34);
    //     assert_dim!("-   12", -12.0);
    //     assert_dim!("+   12", 12.0);
    //     assert_dim!("-  .12", -0.12);
    //     assert_dim!("00.123000", 0.123);
    //     assert_dim!("001.10000", 1.1);
    // }
}
