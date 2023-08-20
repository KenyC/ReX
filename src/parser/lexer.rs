use std::fmt;
use crate::dimensions::{AnyUnit, Unit};
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

    /// The last token which has been lexed.
    current: Token<'a>,
}

impl<'a> Lexer<'a> {
    /// Create a new lexer, whose current token is the first token
    /// to be processed.
    pub fn new(input: &'a str) -> Lexer<'a> {
        let mut lex = Lexer {
            input: input,
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
        self.input = &self.input[size ..];
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
    pub fn dimension(&mut self) -> Option<AnyUnit> {
        fn is_float_char(character : char) -> bool {
            character.is_ascii_digit()
            || character == '-'
            || character == '+'
            || character == ' '
            || character == '.'
        }
        let mut number_to_parse = String::with_capacity(3);
        match self.current {
            Token::Symbol(c) if is_float_char(c) => number_to_parse.push(c),
            _ => return None,
        };

        loop {
            if let Some(c) = self.current_char() {
                if is_float_char(c) {
                    number_to_parse.push(c);
                    self.next_char();
                    continue;
                }
            }
            break;
        }
        let number = number_to_parse.replace(' ', "").parse::<f64>().ok()?;

        self.consume_whitespace();

        // expecting 2 ASCII characters representing the dimension
        let dim = self.input.get(.. 2)?;
        self.advance_by(2);
        self.next();

        match dim {
            "em" => Some(AnyUnit::Em(number)),
            "px" => Some(AnyUnit::Px(number)),
            _ => None
        }
    }

    // Expects to find an {<inner>}, and return <inner>
    pub fn group(&mut self) -> ParseResult<'a, &'a str> {
        self.consume_whitespace();
        self.current.expect(Token::Symbol('{'))?;

        let mut open_parenthesis = 1;
        let mut char_indices_iterator = self.input.char_indices();
        let mut no_escape = true;
        let mut last_index;

        loop {
            let (next_index, next_char) = char_indices_iterator.next().ok_or(ParseError::UnexpectedEof)?;
            last_index = next_index; 
            if no_escape {
                match next_char {
                    '{'  => open_parenthesis += 1,
                    '}'  => open_parenthesis -= 1,
                    '\\' => no_escape = false,
                    _    => (),
                }
            }
            else {
                no_escape = true;
            }

            if open_parenthesis == 0 {
                break;
            }
        }

        // Place cursor immediately after }
        let group_inside = &self.input[.. last_index];
        self.advance_by(last_index);
        self.next();
        Ok(group_inside)
    }

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

    /// Consumes all remaining output till EOF.  
    /// This is useful when another lexer takes over (in the case of custom command), but we still want to make
    /// sure the old one has been consumed.
    pub fn move_to_end(&mut self) -> & 'a str {
        let Self { input, current } = self;
        *current = Token::EOF;
        std::mem::replace(input, &input[0 .. 0])
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

    use crate::dimensions::AnyUnit;

    use super::{Lexer, Token};



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

    #[test]
    fn lex_group() {
        let mut l = Lexer::new("{1}");
        assert_eq!(l.group(), Ok("1"));
        assert!(!(l.current() == Token::EOF));

        let mut l = Lexer::new("   {  abc } ");
        assert_eq!(l.group(), Ok("  abc "));
        assert!(!(l.current() == Token::EOF));

        let mut l = Lexer::new("{}");
        assert_eq!(l.group(), Ok(""));
        assert!(!(l.current() == Token::EOF));


        let mut l = Lexer::new("{fez{fe}}");
        assert_eq!(l.group(), Ok("fez{fe}"));
        assert!(!(l.current() == Token::EOF));

        let mut l = Lexer::new(r"{fez\{}");
        assert_eq!(l.group(), Ok(r"fez\{"));
        assert!(!(l.current() == Token::EOF));


        // This doesn't seem correct:
        // assert_group!("{{}}", Ok("{"));
    }

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

    #[test]
    fn lex_dimension() {
        fn parse_dim(input : &str) -> Option<AnyUnit> {
            let mut lexer = Lexer::new(input);
            lexer.dimension()  
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
