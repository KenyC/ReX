//! Defines all errors that may occur during parsing

use std::fmt;


/// Result type for the [`ParseError`]
pub type ParseResult<T> = ::std::result::Result<T, ParseError>;



/// Syntax error in the formula provided (mismatching brackets, unknown command)
#[derive(Debug, Clone, PartialEq)]
pub enum ParseError {
    /// A macro is missing a required argument.
    RequiredMacroArg,
    /// EOF appeared while a group or an array was incomplete
    UnexpectedEof,
    /// The symbol is not one we have category info about.
    UnrecognizedSymbol(char),
}


impl fmt::Display for ParseError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            ParseError::RequiredMacroArg => 
                write!(f, "missing required macro argument"),
            ParseError::UnexpectedEof =>
                write!(f, "unexpected end of input; unmatched end of array? unfinished group?"),
            ParseError::UnrecognizedSymbol(c) =>
                write!(f, "unrecognized symbol '{}'", c),        
        }
    }
}
