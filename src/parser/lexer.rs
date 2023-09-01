//! Low-level parsing, parsing character, strings, alphanumerics



use std::todo;

use unicode_math::AtomType;

use crate::{dimensions::AnyUnit, RGBA};

use super::{Parser, symbols::Symbol, error::{ParseResult, ParseError}};







impl<'i, 'c> Parser<'i, 'c> {
    /// Advances through the input so that the first character pointed to
    /// is not a whitespace
    pub fn consume_whitespace(&mut self) {
        let mut chars = self.input.chars();

        let mut remainder = self.input;
        while chars.next().map_or(false, |c| c.is_whitespace()) {
            remainder = chars.as_str();
        }

        self.input = remainder;
    }


    /// Attempts parsing a control sequence like `\bla`, returning `bla`.
    pub fn control_sequence<'a>(& 'a mut self) -> Option<& 'a str> {
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


    /// If next char is equal to the argument, advance the input by that character
    /// If not, does nothing.
    pub fn try_parse_char(&mut self, character : char) -> Option<char> {
        let mut chars = self.input.chars();
        if chars.next() == Some(character) {
            self.input = chars.as_str();
            return Some(character)
        }
        None
    }

    /// Gets the next char from the input string (if not empty) and advances the input
    pub fn parse_char(&mut self) -> Option<char> {
        let mut chars = self.input.chars();
        let result = chars.next();
        self.input = chars.as_str();
        result
    }

    /// Gets the next char from the input string (if not empty) and advances the input
    pub fn parse_string(&mut self, string : &str) -> Option<()> {
        self.input.strip_prefix(string)?;
        Some(())
    }

    /// Parses the content of {..} as a plain string
    pub fn parse_group_as_string(&mut self) -> Option<&str> {
        if self.try_parse_char('{').is_some() {
            let (content, remainder) = self.input.split_once('}')?;
            self.input = remainder;
            Some(content)
        }
        else {
            let mut chars = self.input.chars();
            let first_character = chars.next()?;
            let first_character_as_string_slice = &self.input[0 .. first_character.len_utf8()];
            self.input = chars.as_str();
            Some(first_character_as_string_slice)
        }
    }

    /// Parses the input as a dimension, e.g. `1cm` or `-2pt or `3.5em`
    pub fn parse_dimension(mut self) -> ParseResult<AnyUnit> {
        fn is_float_char(character : &char) -> bool {
            character.is_ascii_digit()
            || *character == '-'
            || *character == '+'
            || *character == ' '
            || *character == '.'
        }

        let float_input_to_parse : String = self.input.chars().take_while(is_float_char).collect();
        let number = float_input_to_parse.replace(' ', "").parse::<f64>().map_err(|_| ParseError::ExpectedDimension)?;

        self.input = &self.input[float_input_to_parse.len() ..];
        self.consume_whitespace();

        // expecting 2 ASCII characters representing the dimension
        let dim = self.input.get(.. 2).ok_or_else(|| ParseError::ExpectedDimension)?;

        match dim {
            "em" => Ok(AnyUnit::Em(number)),
            "px" => Ok(AnyUnit::Px(number)),
            _ => Err(ParseError::UnrecognizedDimension),
        }
    } 

    /// Parses a color. A color can be specifed as either:
    ///   1. Alphabetic name for a valid CSS color.
    ///   2. #RRGGBB (that is a # followed by 6 digits)
    ///   3. #RRGGBBAA (that is a # followed by 8 digits)
    pub fn parse_color(mut self) -> ParseResult<RGBA> {
        // If '#' is first character, we have a color specified by value
        if let Some(color_string) = self.input.strip_prefix('#') {
            if color_string.len() == 6 {
                let color = u32::from_str_radix(color_string, 0x10).map_err(|_| todo!())?.to_be_bytes();
                Ok(RGBA(
                    color[1], 
                    color[2], 
                    color[3], 
                    0xff,
                ))
            }
            else if color_string.len() == 8 {
                let color = u32::from_str_radix(color_string, 0x10).map_err(|_| todo!())?.to_be_bytes();
                Ok(RGBA(
                    color[0],
                    color[1], 
                    color[2], 
                    color[3], 
                ))
            }
            else {
                Err(todo!())
            }
        }
        else {
            RGBA::from_name(self.input).ok_or_else(|| ParseError::UnrecognizedColor(self.input.to_string()))
        }

    } 
}

/// Expects an Open or Fence category or a dot
#[inline]
pub fn expect_left(symbol : Symbol) -> ParseResult<Option<Symbol>> {
    if symbol.codepoint == '.' {
        Ok(None)
    }
    else if symbol.atom_type == AtomType::Open || symbol.atom_type == AtomType::Fence {
        Ok(Some(symbol))
    }
    else {
        Err(ParseError::ExpectedOpen(symbol))
    }
}

/// Expects a Fence category or a dot
#[inline]
pub fn expect_middle(symbol : Symbol) -> ParseResult<Option<Symbol>> {
    if symbol.codepoint == '.' {
        Ok(None)
    }
    else if symbol.atom_type == AtomType::Fence {
        Ok(Some(symbol))
    }
    else {
        Err(ParseError::ExpectedMiddle(symbol))
    }
}


/// Expects a Fence or Close category or a dot
#[inline]
pub fn expect_right(symbol : Symbol) -> ParseResult<Option<Symbol>> {
    if symbol.codepoint == '.' {
        Ok(None)
    }
    else if symbol.atom_type == AtomType::Close || symbol.atom_type == AtomType::Fence {
        Ok(Some(symbol))
    }
    else {
        Err(ParseError::ExpectedClose(symbol))
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

    use crate::{dimensions::AnyUnit, parser::{Parser, self, error::ParseResult}, RGBA};


    #[test]
    fn lex_try_char() {
        fn remaining_input(input : &str, character : char) -> (Option<char>, &str) {
            let mut parser = Parser::new(input);
            let outcome = parser.try_parse_char(character);
            (outcome, parser.input)
        }

        assert_eq!(remaining_input("{ rere", '{'), (Some('{'), " rere"));
        assert_eq!(remaining_input("} rere", '{'), (None,     "} rere"));
        assert_eq!(remaining_input("", '{'),       (None,     ""));
    }


    #[test]
    fn lex_primes() {
        todo!()
        // let mut lexer  = Lexer::new("a'b''c'''d");
        // let mut tokens = Vec::new();

        // loop {
        //     let token : Token = todo!();
        //     // let token = lexer.current();
        //     if token == Token::EOF {break;}
        //     tokens.push(token);
        //     todo!();
        //     // lexer.next();
        // }

        // let expected = [
        //     Token::Symbol('a'), 
        //     Token::Command("prime"),
        //     Token::Symbol('b'), 
        //     Token::Command("dprime"),
        //     Token::Symbol('c'), 
        //     Token::Command("trprime"),
        //     Token::Symbol('d'), 
        // ];
        // assert_eq!(
        //     tokens,
        //     expected.to_vec(),
        // )
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
            let mut parser : Parser = Parser::new(input);
            let control_sequence = parser.control_sequence();
            assert_eq!(control_sequence, name);
            assert_eq!(parser.input, remainder);
        }
    }


    #[test]
    fn lex_tokens() {
        todo!()
        // macro_rules! assert_eq_token_stream {
        //     ($left:expr, $right:expr) => {{
        //         let mut left  = Lexer::new($left);
        //         let mut right = Lexer::new($right);

        //         loop {
        //             let l_tok : Token = todo!();
        //             let r_tok : Token = todo!();
        //             // let l_tok = left.next();
        //             // let r_tok = right.next();

        //             assert_eq!(l_tok, r_tok);
        //             if l_tok == Token::EOF {
        //                 break
        //             }
        //         }
        //     }}
        // }

        // assert_eq_token_stream!(r"\cs1", r"\cs 1");
        // assert_eq_token_stream!(r"\cs1", r"\cs    1");
        // assert_eq_token_stream!(r"\cs?", "\\cs\n\n\t?");
        // assert_eq_token_stream!(r"\test\test", r"\test   \test");
        // assert_eq_token_stream!(r"1     +       2", r"1 + 2");
        // assert_eq_token_stream!(r"123\", "123");
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
                todo!()
                // let mut lex = Lexer::new($input);
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
        fn parse_dim(input : &str) -> ParseResult<AnyUnit> {
            let mut parser = Parser::new(input);
            parser.parse_dimension()
        }


        assert_eq!(parse_dim(r"123px abc"),    Ok(AnyUnit::Px(123.0)));
        assert_eq!(parse_dim(r"1.23em abc"),   Ok(AnyUnit::Em(1.23)));
        assert_eq!(parse_dim(r"- 1.23em 123"), Ok(AnyUnit::Em(-1.23)));
        assert_eq!(parse_dim(r"+1.34px 134"),  Ok(AnyUnit::Px(1.34)));
        assert_eq!(parse_dim("-   12em"),      Ok(AnyUnit::Em(-12.0)));
        assert_eq!(parse_dim("+   12px"),      Ok(AnyUnit::Px(12.0)));
        assert_eq!(parse_dim("-  .12em"),      Ok(AnyUnit::Em(-0.12)));
        assert_eq!(parse_dim("00.123000em"),   Ok(AnyUnit::Em(0.123)));
        assert_eq!(parse_dim("001.10000em"),   Ok(AnyUnit::Em(1.1)));

        parse_dim(r"px").unwrap_err();
        parse_dim(r"..em").unwrap_err();
        parse_dim(r"1.4.1em").unwrap_err();

    }


    #[test]
    fn lex_color() {
        fn get_color(input : &str) -> ParseResult<RGBA> {
            Parser::new(input).parse_color()
        }

        assert_eq!(get_color("red"),       Ok(RGBA(0xff, 0x00, 0x00, 0xff)));
        assert_eq!(get_color("darkgray"),  Ok(RGBA(0xa9, 0xa9, 0xa9, 0xff)));
        assert_eq!(get_color("#ffA1e7"),   Ok(RGBA(0xff, 0xa1, 0xe7, 0xff)));
        assert_eq!(get_color("#d25Be84c"), Ok(RGBA(0xd2, 0x5b, 0xe8, 0x4c)));
    }

    #[test]
    fn lex_consume_whitespace() {
        fn remainder_after_consume_whitespace(input : &str) -> &str {
            let mut lexer = Parser::new(input);
            lexer.consume_whitespace();
            lexer.input
        }

        assert_eq!(remainder_after_consume_whitespace("   2"),    "2");
        assert_eq!(remainder_after_consume_whitespace(" \t ç  "), "ç  ");
        assert_eq!(remainder_after_consume_whitespace("  \t  "),  "");
        assert_eq!(remainder_after_consume_whitespace(""),        "");
        assert_eq!(remainder_after_consume_whitespace("abc "),    "abc ");
    }
}
