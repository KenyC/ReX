//! Errors in parsing

use std::fmt;

use super::{control_sequence::PrimitiveControlSequence, GroupKind};


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
    UnrecognizedColor(Box<str>),
    /// A custom macro is missing an argument
    MissingArgForMacro {
        expected : usize,
        got : usize,
    },
    /// The brackets used to enclose a macro's arguments were not matched
    UnmatchedBrackets,
    /// A group (e.g. `{..}`, `\begin{env}..\end{env}`, `&...&`) was ended but there is no correponding begin group
    UnexpectedEndGroup{ 
        expected: Box<[GroupKind]>,
        got:      GroupKind,
    },
    /// A token or group of token was expected but never came
    ExpectedToken,
    /// An argument of control sequence like `\begin{..}` or `\color{..}` must be a sequence of chars ; it cannot contain a command
    ExpectedChars,
    /// A primitive control sequence needs a group as argument but can't find one (e.g. `{\sqrt}+1`).
    MissingArgForCommand(Box<str>),
    /// `\begin{array}` must be followed by a group describing column format (e.g. `\begin{array}{cc}`)
    MissingColFormatForArrayEnvironment,
    /// After `^` and `_`, a group or a token must follow.
    MissingSubSuperScript,
    /// There either is more than one subscript or more than one superscript attached to the same node.
    TooManySubscriptsOrSuperscripts,
    /// The command `\rule` expects an argument of the form `1.3pt` (number followed by dimension). The dimension may not be anything but `em` or `pt` at the moment.
    UnrecognizedDimension(Box<str>),
    /// The string in `\begin{..}` or `\end{..}` is not a recognized environment. Cf [Environment] for the list of supported LaTeX environments.
    UnrecognizedEnvironmen(Box<str>),
    /// The argument of `\begin{array}{..}` is not of the correct form: 
    /// it can only contain the characters `c`, `l`, `r`, whitespaces, braces, `|`  or macros that ultimately expand to one of these.
    UnrecognizedArrayColumnFormat,
    /// The token immediately following `\left`, `\middle` and `\right` isn't a symbol
    ExpectedDelimiter,
    /// The token immediately following `\left` is not of atom type [`AtomType::Open`] or  [`AtomType::Fence`]
    ExpectedOpenDelimiter,
    /// The token immediately following `\middle` is not of atom type [`AtomType::Fence`]
    ExpectedMiddleDelimiter,
    /// The token immediately following `\middle` is not of atom type [`AtomType::Close`]
    ExpectedClosingDelimiter,
    /// The command `\limits` and `\nolimits` must be placed right after an operator (or a macro that expands into something that ends in an operator)
    LimitControlSequenceMustBeAfterOperator,
}


impl fmt::Display for ParseError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        use self::ParseError::*;
        todo!()
    }
}
