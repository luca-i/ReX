//! Errors in parsing

use std::fmt;

use super::GroupKind;


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
    UnexpectedEndGroup(GroupKind),
    /// A token or group of token was expected but never came
    ExpectedToken,
}


impl fmt::Display for ParseError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        use self::ParseError::*;
        todo!()
    }
}
