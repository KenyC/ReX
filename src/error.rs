//! Defines different error types related to various phases of rendering a formula.
//!   - [`FontError`] : errors that have to do with the font file provided (missing MATH table, no such glyph).
//!   - [`ParseError`] : syntax error in the formula provided (mismatching brackets, unknown command).
//!   - [`LayoutError`] : errors during the layout phase ; currently, these can only be font errors.

use crate::font::common::GlyphId;
use crate::parser::lexer::Token;
use std::fmt;
use crate::font::AtomType;
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
    /// Unknown command. \xyz
    UnrecognizedCommand(&'a str),
    /// Unknown environment \begin{xyz}
    UnrecognizedEnvironment(String),
    /// Unknown column specifier \begin{array}{xxx}
    UnrecognizedColumnFormat(Token<'a>),
    /// Unknown vertical alignment argument \begin{array}\[xxx\]{rllc}
    UnrecognizedVerticalAlignmentArg(Token<'a>),
    /// The symbol is not one we have category info about.
    UnrecognizedSymbol(char),
    /// Dimension (e.g. "pt", "cm") ; this error is not thrown at the moment.
    UnrecognizedDimension,
    /// The name of the color used does not appear in `rex::parser::color::COLOR_MAP`
    UnrecognizedColor(String),

    /// unused at present
    ExpectedMathField(Token<'a>),
    /// The first token represents the expected token and the second the token that was obtained.
    ExpectedTokenFound(Token<'a>, Token<'a>),
    /// The symbol after '\left' was not in the "open" or "fence" category.
    ExpectedOpen(Symbol),
    /// The symbol after '\right' was not in the "close" or "fence" category.
    ExpectedClose(Symbol),
    /// The symbol after '\middle' was not in the "fence" category.
    ExpectedMiddle(Symbol),
    /// The first token represents the expected atom type and the second the atom type that was obtained.
    ExpectedAtomType(AtomType, AtomType),
    /// Error thrown if after '\left' and '\right', a token that is not a symbol is found
    ExpectedSymbol(Token<'a>),
    /// Error thrown when parsing a command definition file (cf [`CommandCollection::parse`]), 
    /// when one of the top-level declaration isn't `\newcommand...`. 
    ExpectedNewCommand(Token<'a>),
    /// Error thrown when parsing a command definition file (cf [`CommandCollection::parse`]), 
    /// when the number of arguments ... in `\newcommand{\commandname}[...]` cannot be interpreted as an nonnegative integer 
    ExpectedNumber(String),
    /// Error thrown when parsing a command definition file (cf [`CommandCollection::parse`]), 
    /// when `\newcommand...` isn't followed by the command name (either as `\mycommandname` or `{\mycommandname}`). 
    ExpectedCommandName(Token<'a>),
    /// Expected an opening '{'
    ExpectedOpenGroup,
    /// An unexpected end of environment, e.g. an `\end{bar}' appeared when we were expecting a `\end{foo}'
    UnexpectedEndEnv {
        /// environment expected to end
        expected  : &'a str, 
        /// actual `\end{bar}' implemented
        found : &'a str,     
    },

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
    /// Error thrown when parsing a command definition file (cf [`CommandCollection::parse`]), 
    /// when a command is defined twice
    CommandDefinedTwice(& 'a str),
    /// Error thrown when parsing a command definition file (cf [`CommandCollection::parse`]), 
    /// when the command definition (i.e. the ... in \newcommand{\mycommand}[2]{...}) cannot be parsed.  
    /// This may be because an unescaped # is not followed by a number.
    CannotParseCommandDefinition(& 'a str),
    /// Error thrown when parsing a command definition file (cf [`CommandCollection::parse`]), 
    /// when the command definition (i.e. the ... in \newcommand{\mycommand}[n]{...}) refers to a `#i` with i > n.
    IncorrectNumberOfArguments(usize, usize),

    /// EOF appeared while a group or an array was incomplete
    UnexpectedEof,
    /// Expected EOF because we're done parsing, but there's still some input left.
    /// If this happens, it's a bug of the parser, not a problem in user input.
    ExpectedEof(Token<'a>),


    // TODO: more specific error
    /// Any error happening in the expansion of custom macro  
    /// More specific information about which error it is is currently unsupported 
    /// as it would require some owned strings
    ErrorInMacroExpansion,
    /// An unspecific error value for errors we haven't yet included in the list above
    Todo,
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
        match self {
            UnrecognizedCommand(ref cmd) =>
                write!(f, "unrecognized command: \\{}`", cmd),
            UnrecognizedEnvironment(name) => 
                write!(f, "unrecognized environment: \\begin{{{}}}`", name),
            UnrecognizedColumnFormat(token) => 
                write!(f, "unrecognized column format: {}`", token),
            UnrecognizedVerticalAlignmentArg(token) => 
                write!(f, "unrecognized vertical alignment argument: {}`", token),
            UnrecognizedSymbol(c) =>
                write!(f, "unrecognized symbol '{}'", c),
            FailedToParse(ref tok) =>
                write!(f, "failed to parse `{}`", tok),
            ExcessiveSubscripts =>
                write!(f, "an excessive number of subscripts"),
            ExcessiveSuperscripts =>
                write!(f, "excessive number of superscripts"),
            CommandDefinedTwice(command_name) =>
                write!(f, "'{}' was defined twice", command_name),
            CannotParseCommandDefinition(command_definition) =>
                write!(f, "'{}' cannot be parsed as the body of a command definition", command_definition),
            IncorrectNumberOfArguments(declared, got) =>
                write!(f, "command was declared with {} arguments, but needs at least {} arguments", declared, got),
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
            UnexpectedEndEnv { expected, found } => 
                write!(f, r"expected \end{{{}}}, found \end{{{}}}", expected, found),
            ExpectedNewCommand(ref tok) =>
                write!(f, "expected \newcommand, found {}", tok),
            ExpectedCommandName(ref tok) =>
                write!(f, "expected command name, found {}", tok),
            ExpectedNumber(string) =>
                write!(f, "expected number, found {}", string),
            RequiredMacroArg =>
                write!(f, "missing required macro argument"),
            ExpectedTokenFound(ref expected, ref found) =>
                write!(f, "expected {} found {}", expected, found),
            ExpectedOpen(sym) =>
                write!(f, "expected Open, Fence, or period after '\\left', found `{:?}`", sym),
            ExpectedClose(sym) =>
                write!(f, "expected Close, Fence, or period after '\\right', found `{:?}`", sym),
            ExpectedMiddle(sym) =>
                write!(f, "expected Fence, or period after '\\middle', found `{:?}`", sym),
            ExpectedOpenGroup =>
                write!(f, "expected an open group symbol"),
            NoClosingBracket =>
                write!(f, "failed to find a closing bracket"),
            StackMustFollowGroup =>
                write!(f, "stack commands must follow a group"),
            AccentMissingArg(ref acc) =>
                write!(f, "the accent '\\{}' must have an argument", acc),
            UnexpectedEof =>
                write!(f, "unexpected end of input; unmatched end of array? unfinished group?"),
            ExpectedEof(ref token) =>
                write!(f, "unexpectedly ended parsing; Stoped parsing at {}", token),
            UnrecognizedDimension =>
                write!(f, "failed to parse dimension"),
            UnrecognizedColor(ref color) =>
                write!(f, "failed to recognize the color '{}'", color),
            ErrorInMacroExpansion =>
                write!(f, "some error occurred while expanding a custom command"),
            Todo =>
                write!(f, "failed with an unspecified error that has yet be implemented"),
        }
    }
}
