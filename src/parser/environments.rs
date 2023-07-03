use super::lexer::{Lexer, Token};
use super::macros::CommandCollection;
use crate::font::{Style, AtomType};
use crate::parser::{self, optional_argument_with, required_argument_with, ParseNode, symbols::Symbol};
use crate::error::{ParseResult, ParseError};

/// An enumeration of recognized enviornmnets.
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum Environment {
    Array,
    Matrix,
    PMatrix,
    BMatrix,
    BbMatrix,
    VMatrix,
    VvMatrix,
}

impl Environment {
    /// Attempt to parse an `&str` type into a an `Enviornment`.
    pub fn try_from_str(name: &str) -> Option<Environment> {
        match name {
            "array" => Some(Environment::Array),
            "matrix" => Some(Environment::Matrix),
            "pmatrix" => Some(Environment::PMatrix),
            "bmatrix" => Some(Environment::BMatrix),
            "Bmatrix" => Some(Environment::BbMatrix),
            "vmatrix" => Some(Environment::VMatrix),
            "Vmatrix" => Some(Environment::VvMatrix),
            _ => None,
        }
    }

    /// Parse the enviornment for a given `Environment`.  This can be thought
    /// of as a parsing primitive.
    pub fn parse<'a>(&self, lex: &mut Lexer<'a>, local: Style, command_collection : &CommandCollection) -> ParseResult<'a, ParseNode> {
        match *self {
            Environment::Array    => array(lex, local, command_collection),
            Environment::Matrix   => matrix(lex, local, command_collection),
            Environment::PMatrix  => matrix_with(lex, local, command_collection, '(', ')'),
            Environment::BMatrix  => matrix_with(lex, local, command_collection, '[', ']'),
            Environment::BbMatrix => matrix_with(lex, local, command_collection, '{', '}'),
            Environment::VMatrix  => matrix_with(lex, local, command_collection, '|', '|'),
            Environment::VvMatrix => matrix_with(lex, local, command_collection, '\u{2016}', '\u{2016}'),
        }
    }
}

/// The horizontal positioning of an array.  These are parsed as an optional
/// argument for the Array environment. The default value is `Centered` along
/// the x-axis.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ArrayVerticalAlign {
    /// Centered along the x-axis.
    Centered,

    /// Align the top with the baseline.
    Top,

    /// Align the bottom with the baseline.
    Bottom,
}

impl Default for ArrayVerticalAlign {
    fn default() -> ArrayVerticalAlign {
        ArrayVerticalAlign::Centered
    }
}

// TODO: since we use default values, we should make the argument optional?
/// Array column alignent.  These are parsed as a required macro argument
/// for the array enviornment. The default value is `Centered`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ArrayColumnAlign {
    /// Column is centered
    Centered,

    /// Column is left aligned.
    Left,

    /// Column is right aligned.
    Right,
}

impl Default for ArrayColumnAlign {
    fn default() -> ArrayColumnAlign {
        ArrayColumnAlign::Centered
    }
}

/// Formatting options for a single column.  This includes both the horizontal
/// alignment of the column (clr), and optional vertical bar spacers (on the left).
#[derive(Default, Debug, Clone, Copy, PartialEq, Eq)]
pub struct ArraySingleColumnFormatting {
    /// The alignment of the column.  Defaults to Centered.
    pub alignment: ArrayColumnAlign,

    /// The number of vertical marks before column.
    pub left_vert: u8,
}

/// The collection of column formatting for an array.  This includes the vertical
/// alignment for each column in an array along with optional vertical bars
/// positioned to the right of the last column.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ArrayColumnsFormatting {
    /// The formatting specifications for each column
    pub columns: Vec<ArraySingleColumnFormatting>,

    /// The number of vertical marks after the last column.
    pub right_vert: u8,
}

impl ArrayColumnsFormatting {
    /// Returns center formatting for all columns and no marks
    pub fn default_for(n_cols : usize) -> Self {
        Self { 
            columns:    vec![ArraySingleColumnFormatting::default(); n_cols], 
            right_vert: 0, 
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct Array {
    /// The formatting arguments (clr) for each row.  Default: center.
    pub col_format: ArrayColumnsFormatting,

    /// A collection of rows.  Each row consists of one `Vec<Expression>`.
    pub rows: Vec<Vec<Expression>>,

    /// The left delimiter for the array (optional).
    pub left_delimiter: Option<Symbol>,

    /// The right delimiter for the array (optional).
    pub right_delimiter: Option<Symbol>,
}


fn matrix<'a>(lex: &mut Lexer<'a>, style: Style, command_collection : &CommandCollection) -> ParseResult<'a, ParseNode> {
    matrix_common(lex, style, command_collection, None, None)
}

fn matrix_with<'a>(
    lex: &mut Lexer<'a>,
    style: Style,
    command_collection : &CommandCollection,
    left_delimiter:  char,
    right_delimiter: char
) -> ParseResult<'a, ParseNode> {
    matrix_common(lex, style, command_collection, Some(left_delimiter), Some(right_delimiter))
}

fn matrix_common<'a>(
    lex: &mut Lexer<'a>,
    style: Style,
    command_collection : &CommandCollection,
    left_delimiter:  Option<char>,
    right_delimiter: Option<char>
) -> ParseResult<'a, ParseNode> {
    // matrix bodies are paresed like arrays.
    let body = array_body(lex, style, command_collection)?;
    let left_delimiter = left_delimiter.map(|code| {
                                                Symbol {
                                                    codepoint: code,
                                                    atom_type: AtomType::Inner,
                                                }
                                            });

    let right_delimiter = right_delimiter.map(|code| {
                                                  Symbol {
                                                      codepoint: code,
                                                      atom_type: AtomType::Inner,
                                                  }
                                              });


    let n_cols = body.get(0).map_or(0, |v| v.len());
    Ok(ParseNode::Array(
        Array {
            col_format: ArrayColumnsFormatting::default_for(n_cols),
            rows: body,
            left_delimiter,
            right_delimiter,
        }
    ))
}

/// Parse the column alignments for arrays.  The currently supported formats are:
///   - `c` center the column
///   - `r` right align the column
///   - `l` left align the column
///   - `|` insert a vertical bar at position.
///
/// For example: `\begin{array}{c|c|c}\end{array}`.
fn array_col<'a>(lex: &mut Lexer<'a>, _: Style, _ : &CommandCollection) -> ParseResult<'a, ArrayColumnsFormatting> {
    let mut cols = Vec::new();
    let mut current = ArraySingleColumnFormatting::default();

    loop {
        match lex.current() {
            Token::Symbol('c') => current.alignment = ArrayColumnAlign::Centered,
            Token::Symbol('r') => current.alignment = ArrayColumnAlign::Right,
            Token::Symbol('l') => current.alignment = ArrayColumnAlign::Left,
            Token::Symbol('|') => {
                current.left_vert += 1;
                lex.next();
                lex.consume_whitespace();
                continue;
            }
            Token::Symbol('}') => {
                break;
            }
            token => return Err(ParseError::UnrecognizedColumnFormat(token)),
        }

        cols.push(current);
        current = ArraySingleColumnFormatting::default();

        lex.next();
        lex.consume_whitespace();
    }

    Ok(ArrayColumnsFormatting {
           columns: cols,
           right_vert: current.left_vert,
       })
}

/// Parse the optional argument in an array environment.  This dictates the
/// vertical positioning of the array.  The recognized values are `t` to
/// align the top of the array with the baseline, and `b` to aligne the bottom
/// of the array to the baseline.
///
/// For example: `\begin{array}[t]{cc}..\end{array}`.
fn array_pos<'a>(lex: &mut Lexer<'a>, _: Style) -> ParseResult<'a, Option<ArrayVerticalAlign>> {
    let ret = match lex.current() {
        Token::Symbol('t') => Ok(Some(ArrayVerticalAlign::Top)),
        Token::Symbol('b') => Ok(Some(ArrayVerticalAlign::Bottom)),
        token => return Err(ParseError::UnrecognizedVerticalAlignmentArg(token)),
    };

    lex.next();
    ret
}

/// Array contents are the body of the enviornment.  Columns are seperated
/// by `&` and a newline is terminated by either:
///   - `\\[unit]`
///   - `\cr[unit]`
/// where a `[unit]` is any recognized dimension which will add (or subtract)
/// space between the rows.  Note, the last line termination is ignored
/// if the a line is empty.
type Expression = Vec<ParseNode>;
fn array_body<'a>(lex: &mut Lexer<'a>, style: Style, command_collection : &CommandCollection) -> ParseResult<'a, Vec<Vec<Expression>>> {
    let mut rows: Vec<Vec<Expression>> = Vec::new();
    let mut current: Vec<Expression> = Vec::new();
    loop {
        let expr = parser::expression_until(lex, style, command_collection, Token::Symbol('&'))?;
        if lex.current() == Token::Command(r"end") {
            // If the last line is empty, ignore it.
            if expr.is_empty() && current.is_empty() {
                break;
            }

            current.push(expr);
            rows.push(current);
            break;
        }

        current.push(expr);
        match lex.current() {
            Token::Symbol('&') => { /* no-op, carry on */ }
            Token::Command(r"\") |
            Token::Command(r"cr") => {
                // TODO: Handle space arguments here.
                rows.push(current);
                current = Vec::new();
            }
            _ => return Err(ParseError::UnexpectedEof),
        }
        lex.next();
    }
    Ok(rows)
}

/// Parse an array environment.  This method assumes that the lexer is currently
/// positioned after the `\begin{array}` declaration.
fn array<'a>(lex: &mut Lexer<'a>, local: Style, command_collection : &CommandCollection) -> ParseResult<'a, ParseNode> {
    let pos = optional_argument_with(lex, local, array_pos)?;
    let cols = required_argument_with(lex, local, command_collection, array_col)?;
    let contents = array_body(lex, local, command_collection)?;
    debug!("Array, pos: {:?}, cols: {:?}", pos, cols);
    debug!("Contents: {:#?}", contents);
    Ok(ParseNode::Array(Array {
                            col_format: cols,
                            rows: contents,
                            left_delimiter: None,
                            right_delimiter: None,
                        }))
}
