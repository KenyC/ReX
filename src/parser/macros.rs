//! Structure for custom macros (as created by e.g. `\newcommand{..}`)

use crate::parser::error::ParseError;

use super::{error::ParseResult, textoken::{TexToken, TokenIterator}};




/// A collection of custom commands. You can find a macro with the given name using [`CommandCollection::query`].
#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct CommandCollection(Vec<(String, CustomCommand)>);



impl CommandCollection {
    /// Creates a new empty [`CommandCollection`]    
    pub const fn new() -> Self {
        Self(Vec::new())
    }


    /// Retrieves a method by name
    pub fn get<'s>(& 's self, name : &str) -> Option<& 's CustomCommand> {
        self.0
            .iter()
            .filter_map(|(name_command, command)| if name_command == name { Some(command) } else { None })
            .next()
    }
}




/// A custom LateX command, as defined by e.g. \newcommand.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CustomCommand<'c> {
    n_args : usize,
    // expansion : Vec<TexToken<'c>>
}

impl CustomCommand {
    pub fn n_args(&self) -> usize {
        self.n_args
    }
}


pub struct ExpandedTokenIter<'a, 'c> {
    command_collection : & 'c CommandCollection,
    token_iter : TokenIterator<'a>,
    /// token obtained from macro expansion
    expanded_token : Vec<TexToken<'a>>, 
}

impl<'a, 'c> ExpandedTokenIter<'a, 'c> {
    /// From a regular token iterator, creates one that expands macros.
    pub fn new(command_collection: & 'c CommandCollection, token_iter: TokenIterator<'a>) -> Self {
        Self { command_collection, token_iter, expanded_token: Vec::new() }
    }

    /// Get next token from the iterator
    pub fn next_token(&mut self) -> ParseResult<Option<TexToken<'a>>> {
        if let Some(token) = self.produce_next_token() {
            if let TexToken::ControlSequence(command) = token {
                let Self { command_collection, .. } = self;
                if let Some(command) = command_collection.get(command) {
                    self.expand_macro(command)?;
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

    fn produce_next_token(&mut self) -> Option<TexToken<'a>> {
        Option::or_else(
            self.expanded_token.pop(),
            || self.token_iter.next(),
        )
    }

    fn gather_args_of_command(&mut self, command : &CustomCommand) -> ParseResult<Vec<Vec<TexToken<'a>>>> {
        let n_args = command.n_args();
        let mut args : Vec<Vec<TexToken>> = Vec::with_capacity(n_args);
        for i in 0 .. n_args {
            let mut arg = Vec::with_capacity(1);
            let token = self.produce_next_token()
                .ok_or_else(|| ParseError::MissingArgForMacro {expected : n_args, got : i})?;
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
            else {
                arg.push(token);
            }
            args.push(arg);
        }
        Ok(args)
    }

    fn expand_macro(&mut self, command: &CustomCommand) -> ParseResult<()> {
        let tokens = self.gather_args_of_command(command)?;

        todo!();
        Ok(())
    }
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn check_gather_args() {
        let command = CustomCommand { n_args : 3};
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