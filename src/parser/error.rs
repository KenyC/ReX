//! Defines all errors that may occur during parsing

use std::fmt;

use super::{symbols::Symbol, ParseDelimiter};


/// Result type for the [`ParseError`]
pub type ParseResult<T> = Result<T, ParseError>;



/// Syntax error in the formula provided (mismatching brackets, unknown command)
#[derive(Debug, Clone, PartialEq)]
pub enum ParseError {
    /// A macro is missing a required argument.
    RequiredMacroArg,
    /// EOF appeared while a group or an array was incomplete
    UnexpectedEof,
    /// Unknown environment \begin{xyz}
    UnrecognizedEnvironment(String),
    /// The symbol is not one we have category info about.
    UnrecognizedSymbol(char),


    /// The symbol after '\left' was not in the "open" or "fence" category.
    ExpectedOpen(Symbol),
    /// The symbol after '\right' was not in the "close" or "fence" category.
    ExpectedClose(Symbol),
    /// The symbol after '\middle' was not in the "fence" category.
    ExpectedMiddle(Symbol),
    /// Expected an opening '{'
    ExpectedOpenGroup,
    /// `\left`, `\middle` and `\right` are not followed by a symbol 
    MissingSymbolAfterDelimiter,
    /// `\right` not preceded by `\left`, or separated from it by an open group bracket that isn't closed before
    UnexpectedRight,
    /// `\middle` not preceded by `\left`, or separated from it by an open group bracket that isn't closed before
    UnexpectedMiddle,

    /// Unknown column specifier \begin{array}{xxx}
    UnrecognizedColumnFormat(char),
    /// Expected a delimiter, found another ; for instance, `\right` not preceded by `\left`
    ExpectedDelimiter { 
        /// delimiter found
        found: ParseDelimiter, 
        /// delimiter expected
        expected: ParseDelimiter 
    },
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
            ParseError::UnrecognizedEnvironment(name) => 
                write!(f, "unrecognized environment: \\begin{{{}}}`", name),

            ParseError::ExpectedOpen(sym) =>
                write!(f, "expected Open, Fence, or period after '\\left', found `{:?}`", sym),
            ParseError::ExpectedClose(sym) =>
                write!(f, "expected Close, Fence, or period after '\\right', found `{:?}`", sym),
            ParseError::ExpectedMiddle(sym) =>
                write!(f, "expected Fence, or period after '\\middle', found `{:?}`", sym),
            ParseError::MissingSymbolAfterDelimiter =>
                write!(f, "missing symbol following delimiter"),
            ParseError::UnexpectedRight => 
                write!(f, "unexpected \\right"),
            ParseError::UnexpectedMiddle => 
                write!(f, "unexpected \\middle"),
            ParseError::UnrecognizedColumnFormat(token) => 
                write!(f, "unrecognized column format: {}`", token),
            ParseError::ExpectedDelimiter { found, expected } => 
                write!(f, "`{}` was expected but `{}` was found instead", expected, found),
            ParseError::ExpectedOpenGroup =>
                write!(f, "expected an open group symbol"),

        }
    }
}
