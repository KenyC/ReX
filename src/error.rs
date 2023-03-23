//! Defines different error types related to various phases of rendering a formula.
//!   - [`FontError`] : errors that have to do with the font file provided (missing MATH table, no such glyph).
//!   - [`ParseError`] : syntax error in the formula provided (mismatching brackets, unknown command).
//!   - [`LayoutError`] : errors during the layout phase ; currently, these can only be font errors.

use crate::font::common::GlyphId;
use crate::lexer::Token;
use std::fmt;
use crate::font::{AtomType};
use crate::parser::symbols::Symbol;

/// Result type for the [`LayoutError`]
pub type LayoutResult<T> = ::std::result::Result<T, LayoutError>;
/// Result type for the [`ParseError`]
pub type ParseResult<'a, T> = ::std::result::Result<T, ParseError<'a>>;

/// Errors during the layout phase ; currently, these can only be font errors.
#[derive(Debug, Clone, PartialEq)]
pub enum LayoutError {
    /// a font error
    Font(FontError)
}

/// Errors having to do with font file provided
#[derive(Debug, Clone, PartialEq)]
pub enum FontError {
    /// The font does not contain a glyph for the given char.
    MissingGlyphCodepoint(char),
    /// The font does not contain a glyph with that id.
    MissingGlyphGID(GlyphId),
    /// The font lacks a MATH table.
    NoMATHTable,
}

impl From<FontError> for LayoutError {
    fn from(e: FontError) -> Self {
        LayoutError::Font(e)
    }
}

/// Syntax error in the formula provided (mismatching brackets, unknown command)
#[derive(Debug, Clone, PartialEq)]
pub enum ParseError<'a> {
    /// Unknown command.
    UnrecognizedCommand(&'a str),
    /// The symbol is not one we have category info about.
    UnrecognizedSymbol(char),
    /// Dimension (e.g. "pt", "cm") ; this error is not thrown at the moment.
    UnrecognizedDimension,
    /// The name of the color used does not appear in `rex::parser::color::COLOR_MAP`
    UnrecognizedColor(&'a str),

    /// unused at present
    ExpectedMathField(Token<'a>),
    /// The first token represents the expected token and the second the token that was obtained.
    ExpectedTokenFound(Token<'a>, Token<'a>),
    /// The symbol after '\left' was not in the "open" category.
    ExpectedOpen(Symbol),
    /// The symbol after '\right' was not in the "close" category.
    ExpectedClose(Symbol),
    /// The first token represents the expected atom type and the second the atom type that was obtained.
    ExpectedAtomType(AtomType, AtomType),
    /// Error thrown if after '\left' and '\right', a token that is not a symbol is found
    ExpectedSymbol(Token<'a>),
    /// Expected an opening '{'
    ExpectedOpenGroup,

    /// unused
    MissingSymbolAfterDelimiter,
    /// unused
    MissingSymbolAfterAccent,
    /// The command '\limits' and '\nolimits' can only be used after sums, integrals, limits, ...
    LimitsMustFollowOperator,
    /// A macro is missing a required argument.
    RequiredMacroArg,
    /// A '{' does not find a corresponding '}'.
    NoClosingBracket,

    /// Couldn't find a "{...}" after "\substack"
    StackMustFollowGroup,
    /// unused
    AccentMissingArg(&'a str),
    /// unused
    FailedToParse(Token<'a>),
    /// One has used two or more superscripts in a row, e.g. "x^2^2".
    ExcessiveSubscripts,
    /// One has used two or more subscripts in a row, e.g. "x_2_2".
    ExcessiveSuperscripts,

    /// EOF appeared while some groups were not closed
    UnexpectedEof(Token<'a>),

    /// Our parser does not implement this case yet.
    Todo
}

/// A generic error type covering any error that may happen during the process.
#[derive(Debug, Clone, PartialEq)]
pub enum Error<'a> {
    /// a parse error
    Parse(ParseError<'a>),
    /// a layout error (including font errors)
    Layout(LayoutError)
}
impl<'a> From<ParseError<'a>> for Error<'a> {
    fn from(e: ParseError<'a>) -> Self {
        Error::Parse(e)
    }
}
impl<'a> From<LayoutError> for Error<'a> {
    fn from(e:LayoutError) -> Self {
        Error::Layout(e)
    }
}


impl fmt::Display for FontError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        use self::FontError::*;
        match *self {
            MissingGlyphCodepoint(cp) =>
                write!(f, "missing glyph for codepoint'{}'", cp),
            MissingGlyphGID(gid) =>
                write!(f, "missing glyph with gid {}", Into::<u16>::into(gid)),
            NoMATHTable =>
                write!(f, "no MATH tables"),
        }
    }
}
impl<'a> fmt::Display for ParseError<'a> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        use self::ParseError::*;
        match *self {
            UnrecognizedCommand(ref cmd) =>
                write!(f, "unrecognized command: \\{}`", cmd),
            UnrecognizedSymbol(c) =>
                write!(f, "unrecognized symbol '{}'", c),
            FailedToParse(ref tok) =>
                write!(f, "failed to parse `{}`", tok),
            ExcessiveSubscripts =>
                write!(f, "an excessive number of subscripts"),
            ExcessiveSuperscripts =>
                write!(f, "excessive number of superscripts"),
            LimitsMustFollowOperator =>
                write!(f, "limit commands must follow an operator"),
            ExpectedMathField(ref field) =>
                write!(f, "expected math field, found `{}`", field),
            MissingSymbolAfterDelimiter =>
                write!(f, "missing symbol following delimiter"),
            MissingSymbolAfterAccent =>
                write!(f, "missing symbol following accent"),
            ExpectedAtomType(left, right) =>
                write!(f, "expected atom type {:?} found {:?}", left, right),
            ExpectedSymbol(ref sym) =>
                write!(f, "expected symbol, found {}", sym),
            RequiredMacroArg =>
                write!(f, "missing required macro argument"),
            ExpectedTokenFound(ref expected, ref found) =>
                write!(f, "expected {} found {}", expected, found),
            ExpectedOpen(sym) =>
                write!(f, "expected Open, Fence, or period after '\\left', found `{:?}`", sym),
            ExpectedClose(sym) =>
                write!(f, "expected Open, Fence, or period after '\\right', found `{:?}`", sym),
            ExpectedOpenGroup =>
                write!(f, "expected an open group symbol"),
            NoClosingBracket =>
                write!(f, "failed to find a closing bracket"),
            StackMustFollowGroup =>
                write!(f, "stack commands must follow a group"),
            AccentMissingArg(ref acc) =>
                write!(f, "the accent '\\{}' must have an argument", acc),
            UnexpectedEof(ref tok) =>
                write!(f, "unexpectedly ended parsing; unmatched end of expression? Stoped parsing at {}", tok),
            UnrecognizedDimension =>
                write!(f, "failed to parse dimension"),
            UnrecognizedColor(ref color) =>
                write!(f, "failed to recognize the color '{}'", color),
            Todo =>
                write!(f, "failed with an unspecified error that has yet be implemented"),
        }
    }
}