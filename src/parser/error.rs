//! Errors in parsing

use std::fmt;


/// Result type for the [`ParseError`]
pub type ParseResult<T> = ::std::result::Result<T, ParseError>;


/// Syntax error in the formula provided (mismatching brackets, unknown command)
#[derive(Debug, Clone, PartialEq)]
pub enum ParseError {
    /// The symbol is not one we have atom type info about.
    UnrecognizedSymbol(char),
    /// There is no primitive control sequence with this name
    UnrecognizedControlSequence(Box<str>),
    /// Unable to parse argument of `\color{..}` as a color
    /// Valid color tokens are:
    ///  - Ascii name for css color (ie: `red`).
    ///  - #RRGGBB (ie: `#ff0000` for red)
    ///  - #RRGGBBAA (ie: `#00000000` for transparent)
    ///  - `transparent`
    UnknownColor(Box<str>),
}


impl fmt::Display for ParseError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        use self::ParseError::*;
        todo!()
    }
}
