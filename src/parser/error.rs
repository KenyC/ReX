//! Defines all errors that may occur during parsing

use std::fmt;

use unicode_math::AtomType;

use crate::dimensions::AnyUnit;

use super::{lexer::Token, symbols::Symbol};


/// Result type for the [`ParseError`]
pub type ParseResult<T> = ::std::result::Result<T, ParseError>;



/// Syntax error in the formula provided (mismatching brackets, unknown command)
#[derive(Debug, Clone, PartialEq)]
pub enum ParseError {

}


impl fmt::Display for ParseError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        // match self {

        // }
        Ok(todo!())
    }
}
