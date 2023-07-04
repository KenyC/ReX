//! Structure for custom macros (as created by e.g. `\newcommand{..}`)

use std::unreachable;

use crate::{error::ParseError, parser::lexer::{Lexer, Token}};


/// A collection of custom commands. You can find a macro with the given name using [`CommandCollection::query`].
#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct CommandCollection(Vec<(String, CustomCommand)>);



impl CommandCollection {
    /// Returns a reference to the macro with the given name, if there is a macro with that name
    pub fn query(&self, name : &str) -> Option<&CustomCommand> {
        for (macro_name, custom_cmd) in self.0.iter() {
            if macro_name == name {
                return Some(custom_cmd);
            }
        }
        None
    }

    // TODO: is failure the right behavior here? rather than overwrite
    /// Inserts a new command into the collection.  
    /// If a command of that name already exists, the insertion fails.
    pub fn insert(&mut self, name : &str, command : CustomCommand) -> Option<()> {
        if self.query(name).is_none() {
            self.0.push((name.to_string(), command));
            Some(())
        }
        else {
            None
        }
    }

    /// Parse a series of `\newcommand{...}[]{fezefzezf}` into a command collection
    pub fn parse(command_definitions : &str) -> Result<Self, ParseError> {
        let mut lexer = Lexer::new(command_definitions);
        let mut to_return = CommandCollection::default();


        while lexer.current() != Token::EOF {
            match lexer.current() {
                Token::Command("newcommand") => (),
                Token::Symbol(_) | Token::Command(_) => 
                    return Err(ParseError::ExpectedNewCommand(lexer.current())),
                Token::WhiteSpace => {
                    lexer.consume_whitespace();
                    continue;
                },
                Token::EOF => unreachable!("already checked above"),
            };


            // -- parse command name
            lexer.consume_whitespace();
            lexer.next();
            let token_command_name;
            if let Ok(inner) = lexer.group() {
                let mut lexer = Lexer::new(inner);
                lexer.consume_whitespace();
                token_command_name = lexer.current();
            }
            else {
                token_command_name = lexer.current();
            }

            let command_name = match token_command_name {
                Token::Command(name) => name,
                tok => return Err(ParseError::ExpectedCommandName(tok)),
            };

            // -- parse number of arguments
            lexer.next().expect_symbol('[')?;
            lexer.next();
            let alphanumeric = lexer.alphanumeric();
            let n_args = alphanumeric.parse::<usize>().ok().ok_or(ParseError::ExpectedNumber(alphanumeric))?;
            lexer.current().expect_symbol(']')?;


            // -- parse definition body
            lexer.next();
            let definition = lexer.group()?;

            // TODO : more specific error message?
            let custom_command = CustomCommand::parse(definition).ok_or(ParseError::CannotParseCommandDefinition(definition))?;

            if custom_command.n_args != n_args {
                return Err(ParseError::IncorrectNumberOfArguments(custom_command.n_args, n_args));
            }
            to_return.insert(command_name, custom_command);


            lexer.next();
        }

        Ok(to_return)
    }
}




/// A custom LateX command, as defined by e.g. \newcommand.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CustomCommand {
    n_args  : usize,
    chunks  : Vec<ChunkCommand>,
}

impl CustomCommand {
    /// Parse a string of the form "... #1 ... #23 .. #45" containing pounds followed by numbers
    /// Retrieves the argument numbers and the string in between each pound-number combination
    pub fn parse(body : &str) -> Option<Self> {
        let mut chunks : Vec<ChunkCommand> = Vec::new();
        let mut current_string_start = 0;
        enum ParseState {
            ReadStringEscape, 
            ReadNumber,
            ReadString,
        }
        use ParseState::*;
        let mut state = ReadString;
        let mut arg_no_max = 0;

        for (index, character) in body.char_indices() {
            state = match (state, character) {
                (ReadStringEscape, _) => ReadString,
                (ReadString, '\\')    => ReadStringEscape,
                (ReadString, '#')     => {
                    if index != current_string_start {
                        chunks.push(ChunkCommand::Text(String::from(&body[current_string_start .. index])));
                    }
                    current_string_start = index + '#'.len_utf8();
                    ReadNumber
                },
                (ReadNumber, c) if !c.is_ascii_digit() => {
                    if index != current_string_start {
                        let arg_no = body[current_string_start .. index].parse::<usize>().ok()?;
                        if arg_no > arg_no_max { arg_no_max = arg_no; }
                        // LaTeX's args are one-indexed, we prefer zero-indexing
                        chunks.push(ChunkCommand::ArgSlot(arg_no - 1));
                        current_string_start = index;
                        match c {
                            '\\' => ReadStringEscape,
                            '#'  => {current_string_start += '#'.len_utf8(); ReadNumber},
                            _    => ReadString
                        }
                    }
                    else {
                        return None;
                    }
                }
                (s, _) => s,
            };
        }

        match state {
            ReadString | ReadStringEscape => {
                chunks.push(ChunkCommand::Text(String::from(&body[current_string_start .. ])));
            }
            ReadNumber if body.len() == current_string_start => return None,
            ReadNumber => {
                let arg_no = body[current_string_start .. ].parse::<usize>().ok()?;
                if arg_no > arg_no_max { arg_no_max = arg_no; }
                // LaTeX's args are one-indexed, we prefer zero-indexing
                chunks.push(ChunkCommand::ArgSlot(arg_no - 1));
            }
            _ => (),
        }

        
        Some(Self { 
            n_args: arg_no_max, 
            chunks, 
        })
    }


    /// This method returns the string obtained by replacing the argument slots with the provided values.
    /// This method does not check if the number of arguments given is correct and may panic if provided with too few arguments
    pub fn apply(&self, args : &[&str]) -> String {
        // This string is added on both sides of arguments to prevent 
        // expansion like "\wrapbraces{a}" => "\lbracea\brace" (syntax error)
        // With guards, the macro is expanded as "\lbrace{}a{}\rbrace"
        const GUARD: &str = "{}";
        const ARG_GUARD_SIZE:   usize = 2 * GUARD.len();
        const FINAL_GUARD_SIZE: usize = GUARD.len();
        let string_size : usize = self.chunks
            .iter()
            .map(|chunk| match chunk {
                ChunkCommand::ArgSlot(i) => args[*i].len() + ARG_GUARD_SIZE,
                ChunkCommand::Text(text) => text.len(),
            })
            .sum::<usize>()
            + FINAL_GUARD_SIZE
        ;

        let mut to_return = String::with_capacity(string_size);

        for chunk in self.chunks.iter() {
            match chunk {
                ChunkCommand::ArgSlot(i) => {
                    to_return.push_str(GUARD);
                    to_return.push_str(args[*i]);
                    to_return.push_str(GUARD);
                },
                ChunkCommand::Text(text) => to_return.push_str(text),
            };
        }
        to_return.push_str(GUARD);
        to_return
    }


    /// Number of arguments required by command
    pub fn n_args(&self) -> usize {
        self.n_args
    }
}




#[derive(Debug, Clone, PartialEq, Eq)]
enum ChunkCommand {
    ArgSlot(usize),
    Text(String),
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_custom_command() {
        use super::ChunkCommand::*;
        let command_def = "I love #1 and 2";
        let expected = Some(CustomCommand {
            n_args: 1,
            chunks: vec![Text("I love ".to_string()), ArgSlot(0), Text(" and 2".to_string()),],
        });
        assert_eq!(CustomCommand::parse(command_def), expected);

        let command_def = "#45#1";
        let expected = Some(CustomCommand {
            n_args: 45,
            chunks: vec![ArgSlot(44), ArgSlot(0),],
        });
        assert_eq!(CustomCommand::parse(command_def), expected);
    }

    #[test]
    fn apply_custom_command() {
        use super::ChunkCommand::*;
        let command_def = "I love #1 and 2";
        let expected = CustomCommand {
            n_args: 1,
            chunks: vec![Text("I love ".to_string()), ArgSlot(0), Text(" and 2".to_string()),],
        };
        let custom_command = CustomCommand::parse(command_def).unwrap();
        assert_eq!(custom_command, expected);
        let result = custom_command.apply(&["custard",]);
        assert_eq!(result, "I love {}custard{} and 2{}");

        let command_def = r"\left\lbrace #1\middle| #2\right\rbrace";
        let custom_command = CustomCommand::parse(command_def).unwrap();
        let result = custom_command.apply(&["x + 2", "x"]);
        assert_eq!(result, r"\left\lbrace {}x + 2{}\middle| {}x{}\right\rbrace{}");
    }

    #[test]
    fn parse_command_file() {
        use super::ChunkCommand::*;

        let file = include_str!("macros_test_files/ok1.tex");

        let expected = CommandCollection(vec![
            ("dbb".to_string(), CustomCommand { n_args : 1, chunks: vec![
                Text(r"\left\lBrack".to_string()),
                ArgSlot(0),
                Text(r"\right\rBrack".to_string()),
            ]}),
            ("quo".to_string(), CustomCommand { n_args : 1, chunks: vec![
                Text(r"``\mathrm{".to_string()),
                ArgSlot(0),
                Text(r"}''".to_string()),
            ]}),
            ("poly".to_string(), CustomCommand { n_args : 3, chunks: vec![
                ArgSlot(0),
                Text(r"x^2 + ".to_string()),
                ArgSlot(1),
                Text(r"x + ".to_string()),
                ArgSlot(2),
                Text(r" = 0".to_string()),
            ]})
        ]);
        let got = CommandCollection::parse(file).unwrap();
        assert_eq!(expected, got);


        let file = include_str!("macros_test_files/ok2.tex");

        let expected = CommandCollection(vec![
            ("dbb".to_string(), CustomCommand { n_args : 1, chunks: vec![
                Text(r"\left\lBrack".to_string()),
                ArgSlot(0),
                Text(r"\right\rBrack".to_string()),
            ]}),
            ("poly".to_string(), CustomCommand { n_args : 3, chunks: vec![
                ArgSlot(0),
                Text(r"x^2 + ".to_string()),
                ArgSlot(1),
                Text(r"x + ".to_string()),
                ArgSlot(2),
                Text(r" = 0".to_string()),
            ]}),
        ]);
        let got = CommandCollection::parse(file).unwrap();
        assert_eq!(expected, got);


    }
}