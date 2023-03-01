
use crate::font::common::GlyphId;
use crate::lexer::Token;
use std::fmt;
use crate::font::{AtomType};
use crate::parser::symbols::Symbol;

pub type LayoutResult<T> = ::std::result::Result<T, LayoutError>;
pub type ParseResult<'a, T> = ::std::result::Result<T, ParseError<'a>>;

#[derive(Debug, Clone, PartialEq)]
pub enum LayoutError {
    Font(FontError)
}

#[derive(Debug, Clone, PartialEq)]
pub enum FontError {
    MissingGlyphCodepoint(char),
    MissingGlyphGID(GlyphId),
    NoMATHTable,
}

impl From<FontError> for LayoutError {
    fn from(e: FontError) -> Self {
        LayoutError::Font(e)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum ParseError<'a> {
    UnrecognizedCommand(&'a str),
    UnrecognizedSymbol(char),
    UnrecognizedDimension,
    UnrecognizedColor(&'a str),

    ExpectedMathField(Token<'a>),
    ExpectedTokenFound(Token<'a>, Token<'a>),
    ExpectedOpen(Symbol),
    ExpectedClose(Symbol),
    ExpectedAtomType(AtomType, AtomType),
    ExpectedSymbol(Token<'a>),
    ExpectedOpenGroup,

    MissingSymbolAfterDelimiter,
    MissingSymbolAfterAccent,
    LimitsMustFollowOperator,
    RequiredMacroArg,
    NoClosingBracket,
    StackMustFollowGroup,
    AccentMissingArg(&'a str),
    FailedToParse(Token<'a>),
    ExcessiveSubscripts,
    ExcessiveSuperscripts,

    UnexpectedEof(Token<'a>),

    Todo
}
#[derive(Debug, Clone, PartialEq)]
pub enum Error<'a> {
    Parse(ParseError<'a>),
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