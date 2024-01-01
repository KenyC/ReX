//! Defines all errors that may occur during parsing

use std::fmt;

use unicode_math::AtomType;

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
    /// Unknown command. \xyz
    UnrecognizedCommand(String),
    /// Dimension (e.g. "pt", "cm") ; this error is not thrown at the moment.
    UnrecognizedDimension,
    /// The symbol is not one we have category info about.
    UnrecognizedSymbol(char),
    /// Either the name of the color used does not appear in `rex::parser::color::COLOR_MAP` or the RGBA value can't be parsed as hexadecimal
    UnrecognizedColor(String),


    /// The symbol after '\left' was not in the "open" or "fence" category.
    ExpectedOpen(Symbol),
    /// The symbol after '\right' was not in the "close" or "fence" category.
    ExpectedClose(Symbol),
    /// The symbol after '\middle' was not in the "fence" category.
    ExpectedMiddle(Symbol),
    /// Expected an opening '{'
    ExpectedOpenGroup,
    /// After `\rule`, a number followed by a dimension was expected
    ExpectedDimension,
    /// The first token represents the expected atom type and the second the atom type that was obtained.
    ExpectedAtomType { 
        /// expected atom type
        expected : AtomType, 
        /// atom type encountered
        found : AtomType 
    },
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


    /// One node has two or more superscripts in a row, e.g. "x^2^2".
    ExcessiveSubscripts,
    /// One node has two or more subscripts in a row, e.g. "x_2_2".
    ExcessiveSuperscripts,
    CommandDoesNotStartWithBackslash,


    // -- Related to custom command definitions ; cf [CustomCollection]

    /// Expected a `\newcommand` control sequence, either didn't get a `newcommand` (`None`) 
    /// or got another control sequence (`Some(name_of_other_control_seq)`)
    ExpectedNewCommand(Option<String>),
    /// After `\newcommand`, the parser expects a group of the form `{ \mycommandname }` 
    /// indicating the name of the command to be added
    ExpectedCommandName,
    /// After `\newcommand{\mycommandname}`, the parser expects an argument of form `[ .. 4 ..]` 
    /// indicating the number of arguments taken by the custom command
    ExpectedNumber(String),
    /// After `\newcommand{\mycommandname}[4]`, the parser expects group `{ .. 4 ..}` containing the definition of the command 
    ExpectedCommandDefinition,
    /// The body of the command could not be parsed
    CannotParseCommandDefinition,
}


impl fmt::Display for ParseError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        use self::ParseError::*;
        match self {
            RequiredMacroArg => 
                write!(f, "missing required macro argument"),
            UnexpectedEof =>
                write!(f, "unexpected end of input; unmatched end of array? unfinished group?"),
            UnrecognizedCommand(cmd) =>
                write!(f, "unrecognized command: \\{}`", cmd),
            UnrecognizedSymbol(c) =>
                write!(f, "unrecognized symbol '{}'", c),
            UnrecognizedDimension =>
                write!(f, "failed to parse dimension"),        
            UnrecognizedEnvironment(name) => 
                write!(f, "unrecognized environment: \\begin{{{}}}`", name),
            UnrecognizedColor(ref color) =>
                write!(f, "failed to recognize the color '{}'", color),

            ExpectedOpen(sym) =>
                write!(f, "expected Open, Fence, or period after '\\left', found `{:?}`", sym),
            ExpectedClose(sym) =>
                write!(f, "expected Close, Fence, or period after '\\right', found `{:?}`", sym),
            ExpectedMiddle(sym) =>
                write!(f, "expected Fence, or period after '\\middle', found `{:?}`", sym),
            ExpectedAtomType { expected, found } =>
                write!(f, "expected atom type {:?} found {:?}", expected, found),
            ExpectedDimension => 
                write!(f, "expected dimension"),
            ExpectedDelimiter { found, expected } => 
                write!(f, "`{}` was expected but `{}` was found instead", expected, found),
            ExpectedOpenGroup =>
                write!(f, "expected an open group symbol"),
                
            MissingSymbolAfterDelimiter =>
                write!(f, "missing symbol following delimiter"),
            UnexpectedRight => 
                write!(f, "unexpected \\right"),
            UnexpectedMiddle => 
                write!(f, "unexpected \\middle"),
            UnrecognizedColumnFormat(token) => 
                write!(f, "unrecognized column format: {}`", token),

            ExcessiveSubscripts =>
                write!(f, "excessive number of subscripts"),
            ExcessiveSuperscripts =>
                write!(f, "excessive number of superscripts"),

            ExpectedNewCommand(Some(name)) =>
                write!(f, "expected control `newcommand` control sequence, got `{}` instead", name),
            ExpectedNewCommand(None) =>
                write!(f, "expected control `newcommand` control sequence, didn't get a control sequence"),
            CommandDoesNotStartWithBackslash => 
                write!(f, "the new command name did not start with a backslash"),
            ExpectedCommandName => 
                write!(f, "first argument of `\\newcommand` cannot be parsed as the name of a command"),

            ExpectedNumber(string) => 
                write!(f, "expected integer as second argument of `\\newcommand`, got `{}`", string),
            ExpectedCommandDefinition => 
                write!(f, "expected command definition as third argument of `\\newcommand`"),
            CannotParseCommandDefinition => 
                write!(f, "could not parse command definition"),

        }
    }
}
