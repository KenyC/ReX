//! Structure for custom macros (as created by e.g. `\newcommand{..}`)

use std::pin::Pin;

use crate::parser::error::ParseError;

use super::{error::ParseResult, textoken::{TexToken, TokenIterator}};




/// A collection of custom commands. You can find a macro with the given name using [`CommandCollection::query`].
#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct CommandCollection(Vec<CustomCommand>);



impl CommandCollection {
    /// Creates a new empty [`CommandCollection`]    
    pub const fn new() -> Self {
        Self(Vec::new())
    }


    /// Retrieves a method by name
    pub fn get<'s>(& 's self, name : &str) -> Option<& 's CustomCommand> {
        self.0
            .iter()
            .find(|command| command.name() == name)
    }
}


#[derive(Debug, Clone, PartialEq, Eq)]
enum CommandToken {
    NormalToken(TexToken<'static>),
    OwnedCommand(String),
    ArgSlot(usize),
}

/// A custom LateX command, as defined by e.g. \newcommand.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CustomCommand {
    n_args : usize,
    name : String,

    // !! This should all be private
    expansion : Vec<CommandToken>,
}

struct ExpansionIterator<'args, 'token> {
    token_remaining : & 'token [CommandToken],
    args : & 'args [Vec<TexToken<'token>>],
    arg_currently_output : Option<& 'args [TexToken<'token>]>,
}

impl<'arg, 'a> Iterator for ExpansionIterator<'arg, 'a> {
    type Item = TexToken<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        let Self { token_remaining, args, arg_currently_output } = self;
        let (first_token, rest) = token_remaining.split_first()?;
        match first_token {
            CommandToken::NormalToken(token) => {*token_remaining = rest; Some(token.clone())},
            CommandToken::OwnedCommand(name) => {*token_remaining = rest; Some(TexToken::ControlSequence(name))},
            CommandToken::ArgSlot(i)         => 
                if let Some(tokens) = arg_currently_output {
                    if let Some((first_token, rest)) = tokens.split_first() {
                        arg_currently_output.replace(rest);
                        Some(first_token.clone())
                    }
                    else {
                        *arg_currently_output = None;
                        *token_remaining = rest;
                        self.next()
                    }
                }
                else {
                    *arg_currently_output = Some(args[*i].as_slice());
                    self.next()
                }
            ,
        }
    }
}



impl CustomCommand {
    pub fn empty_command(name : &str, n_args : usize) -> Self {
        Self { n_args, name: name.to_string(), expansion: Vec::new() }
    }

    pub fn n_args(&self) -> usize {
        self.n_args
    }

    fn expand_iter<'args, 'token>(& 'token self, args : & 'args [Vec<TexToken<'token>>]) -> ExpansionIterator<'args, 'token> {
        let Self { expansion, .. } = self;
        ExpansionIterator { 
            token_remaining: expansion.as_slice(), 
            args, 
            arg_currently_output: None,
        }
    }
    // fn expand<'a, 'arg, 't>(& 'a self, args : & 'arg [Vec<TexToken<'t>>]) -> ExpansionIterator<'a, 'arg, 't> {
    //     todo!()
    // }

    pub fn name(&self) -> &str {
        &self.name
    }
}


pub struct ExpandedTokenIter<'d, 'a : 'd, 'c : 'd> {
    command_collection : & 'c CommandCollection,
    token_iter : TokenIterator<'a>,
    /// token obtained from macro expansion
    expanded_token : Vec<TexToken<'d>>, 
}

impl<'d, 'a : 'd, 'c : 'd> Iterator for ExpandedTokenIter<'d, 'a, 'c> {
    type Item = TexToken<'d>;

    fn next(&mut self) -> Option<Self::Item> {
        self.next_token().ok().flatten()
    }
}

impl<'d, 'a : 'd, 'c : 'd> ExpandedTokenIter<'d, 'a, 'c> {

    /// Get next token from the iterator
    pub fn next_token(&mut self) -> ParseResult<Option<TexToken<'d>>> {
        if let Some(token) = self.produce_next_token() {
            if let TexToken::ControlSequence(command) = token {
                let Self { command_collection, .. } = self;
                if let Some(command) = command_collection.get(command) {
                    let tokens: Vec<Vec<TexToken<'d>>> = self.gather_args_of_command(command)?;
                    let token_slice : & [Vec<TexToken<'d>>] = tokens.as_slice();
                    // TODO: something not to have to do reversals
                    let mut expanded_tokens : Vec<TexToken<'d>> = command.expand_iter(token_slice).collect();
                    self.expanded_token.reserve(expanded_tokens.len());
                    while let Some(token) = expanded_tokens.pop() {
                        self.expanded_token.push(token)
                    }
                    self.next_token()
                }
                else {
                    Ok(Some(token))
                }
            }
            else {
                Ok(Some(token))
            }
        }
        else {
            Ok(None)            
        }
    }

    /// From a regular token iterator, creates one that expands macros.
    pub fn new(command_collection: & 'c CommandCollection, token_iter: TokenIterator<'a>) -> Self {
        Self { command_collection, token_iter, expanded_token: Vec::new() }
    }


    fn produce_next_token(&mut self) -> Option<TexToken<'d>> {
        Option::or_else(
            self.expanded_token.pop(),
            || self.token_iter.next(),
        )
    }

    fn gather_args_of_command(&mut self, command : &CustomCommand) -> ParseResult<Vec<Vec<TexToken<'d>>>> {
        let n_args = command.n_args();
        let mut args : Vec<Vec<TexToken>> = Vec::with_capacity(n_args);
        for i in 0 .. n_args {
            let mut arg = Vec::with_capacity(1);
            let mut token = self.produce_next_token()
                .ok_or_else(|| ParseError::MissingArgForMacro {expected : n_args, got : i})?;

            // when looking for arguments of a macro, skip whitespaces
            while token.is_whitespace() {
                token = self.produce_next_token()
                    .ok_or_else(|| ParseError::MissingArgForMacro {expected : n_args, got : i})?;
            }

            // begingroup, we must collect all tokens till the corresponding endgroup token
            if token.is_begin_group() {
                let mut n_open_paren : u32 = 1;
                while n_open_paren != 0 {
                    let token = self.produce_next_token()
                        .ok_or(ParseError::UnmatchedBrackets)?;
                    if token.is_begin_group() {
                        n_open_paren += 1;
                    }
                    else if token.is_end_group() {
                        n_open_paren -= 1;
                    }

                    arg.push(token);
                }
                arg.pop(); // the last bracket shouldn't be added
            }
            // otherwise the given token is the argument
            else {
                arg.push(token);
            }
            args.push(arg);
        }
        Ok(args)
    }

}


#[cfg(test)]
mod tests {
    use super::*;

    impl CustomCommand {
        fn test_command1(name : &str) -> Self {
            let expansion = vec![
                CommandToken::NormalToken(TexToken::Char('x')),
                CommandToken::ArgSlot(0),
                CommandToken::NormalToken(TexToken::Char('y')),
                CommandToken::ArgSlot(1),
                CommandToken::OwnedCommand("cmd".to_string()),
                CommandToken::NormalToken(TexToken::Char('z')),
                CommandToken::ArgSlot(0),
            ];
            Self {
                n_args: 2,
                name: name.to_string(),
                expansion,
            }          
        }

        fn test_command2(name : &str) -> Self {
            let expansion = vec![
                CommandToken::NormalToken(TexToken::Char('x')),
                CommandToken::ArgSlot(0),
                CommandToken::NormalToken(TexToken::Char('y')),
                CommandToken::ArgSlot(1),
                CommandToken::NormalToken(TexToken::Char('z')),
            ];
            Self {
                n_args: 2,
                name: name.to_string(),
                expansion,
            }          
        }

        fn test_command3(name : &str) -> Self {
            let expansion = vec![
                CommandToken::NormalToken(TexToken::Char('o')),
                CommandToken::ArgSlot(0),
                CommandToken::NormalToken(TexToken::Char('c')),
            ];
            Self {
                n_args: 1,
                name: name.to_string(),
                expansion,
            }          
        }
    }

    impl CommandCollection {
        fn test_collection() -> Self {
            Self(vec![
                CustomCommand::test_command1("testone"), 
                CustomCommand::test_command2("testtwo"), 
                CustomCommand::test_command3("testtri"), 
            ])
        }
    }

    #[test]
    fn check_expansion_iterator() {
        let command_collection = CommandCollection::test_collection();

        let input_iterator  = TokenIterator::new(r"a\testtwo{b}{c}d");
        let output_iterator = ExpandedTokenIter::new(&command_collection, input_iterator);
        let tokens : Vec<_> = output_iterator.collect();
        let expected = vec![
            TexToken::Char('a'),
            TexToken::Char('x'),
            TexToken::Char('b'),
            TexToken::Char('y'),
            TexToken::Char('c'),
            TexToken::Char('z'),
            TexToken::Char('d'),
        ];
        assert_eq!(tokens, expected);
    
        let input_iterator  = TokenIterator::new(r"a\testtwo bcd");
        let output_iterator = ExpandedTokenIter::new(&command_collection, input_iterator);
        let tokens : Vec<_> = output_iterator.collect();
        assert_eq!(tokens, expected);

        let input_iterator  = TokenIterator::new(r"a\testtwo{b}cd");
        let output_iterator = ExpandedTokenIter::new(&command_collection, input_iterator);
        let tokens : Vec<_> = output_iterator.collect();
        assert_eq!(tokens, expected);

        let input_iterator  = TokenIterator::new(r"a\testtwo{b} cd");
        let output_iterator = ExpandedTokenIter::new(&command_collection, input_iterator);
        let tokens : Vec<_> = output_iterator.collect();
        assert_eq!(tokens, expected);

        let input_iterator  = TokenIterator::new(r"a\testtri{\testtri b}c");
        let output_iterator = ExpandedTokenIter::new(&command_collection, input_iterator);
        let tokens : Vec<_> = output_iterator.collect();
        let expected = vec![
            TexToken::Char('a'),
            TexToken::Char('o'),
            TexToken::Char('o'),
            TexToken::Char('b'),
            TexToken::Char('c'),
            TexToken::Char('c'),
            TexToken::Char('c'),
        ];
        assert_eq!(tokens, expected);

    }

    #[test]
    fn check_macro_expansion() {
        let args = vec![
            vec![TexToken::Char('a'), TexToken::Char('b'),],
            vec![TexToken::Char('c'), TexToken::Char('d'),],
        ];
        let command = CustomCommand::test_command1("test");

        let expected = vec![
            TexToken::Char('x'),
            TexToken::Char('a'), TexToken::Char('b'),
            TexToken::Char('y'),
            TexToken::Char('c'), TexToken::Char('d'),
            TexToken::ControlSequence("cmd"),
            TexToken::Char('z'),
            TexToken::Char('a'), TexToken::Char('b'),
        ];

        let result : Vec<_> = command.expand_iter(&args).collect();
        assert_eq!(result, expected);
    
        let args = vec![
            vec![],
            vec![TexToken::Char('c'), TexToken::Char('d'),],
        ];
        let expected = vec![
            TexToken::Char('x'),
            TexToken::Char('y'),
            TexToken::Char('c'), TexToken::Char('d'),
            TexToken::ControlSequence("cmd"),
            TexToken::Char('z'),
        ];

        let result : Vec<_> = command.expand_iter(&args).collect();
        assert_eq!(result, expected);
    }


    #[test]
    fn check_gather_args() {
        let command = CustomCommand::empty_command("bla", 3);
        let collection = CommandCollection::new();

        let underlying_string = "abc";
        let token_iter = TokenIterator::new(underlying_string);

        let mut expanded_token_iter = ExpandedTokenIter::new(&collection, token_iter);
        let arg_commands = expanded_token_iter.gather_args_of_command(&command);
        assert!(arg_commands.is_ok());
        let arg_commands = arg_commands.unwrap();
        assert!(arg_commands.len() == 3);


        let underlying_string = "{a}b{c}";
        let token_iter = TokenIterator::new(underlying_string);

        let mut expanded_token_iter = ExpandedTokenIter::new(&collection, token_iter);
        let arg_commands = expanded_token_iter.gather_args_of_command(&command);
        assert!(arg_commands.is_ok());
        let arg_commands = arg_commands.unwrap();
        assert!(arg_commands.len() == 3);

        let underlying_string = "{{a}b}{c}d";
        let token_iter = TokenIterator::new(underlying_string);

        let mut expanded_token_iter = ExpandedTokenIter::new(&collection, token_iter);
        let arg_commands = expanded_token_iter.gather_args_of_command(&command);
        assert!(arg_commands.is_ok());
        let arg_commands = arg_commands.unwrap();
        assert!(arg_commands.len() == 3);


        let underlying_string = r"{{a}b\}c}de";
        let token_iter = TokenIterator::new(underlying_string);

        let mut expanded_token_iter = ExpandedTokenIter::new(&collection, token_iter);
        let arg_commands = expanded_token_iter.gather_args_of_command(&command);
        assert!(arg_commands.is_ok());
        let arg_commands = arg_commands.unwrap();
        assert!(arg_commands.len() == 3);



        // ERRORED INPUT
        let underlying_string = "{abc";
        let token_iter = TokenIterator::new(underlying_string);

        let mut expanded_token_iter = ExpandedTokenIter::new(&collection, token_iter);
        let arg_commands = expanded_token_iter.gather_args_of_command(&command);
        assert!(arg_commands.is_err());

        let underlying_string = "{ab}c";
        let token_iter = TokenIterator::new(underlying_string);

        let mut expanded_token_iter = ExpandedTokenIter::new(&collection, token_iter);
        let arg_commands = expanded_token_iter.gather_args_of_command(&command);
        assert!(arg_commands.is_err());


    }
}