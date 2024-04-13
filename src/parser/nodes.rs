//! Nodes are the output of parsing.

use crate::dimensions::AnyUnit;
use crate::layout::Style;
use super::color::RGBA;
use crate::font::AtomType;
use super::symbols::Symbol;

/// Nodes are the output of parsing a LateX formula ; they can then be arranged in space with [`crate::layout::engine::layout`].
// TODO: It might be worth letting the `Group` variant
//   to have an atomtype associated with it.  By default,
//   it will be a `Ordinary`.
#[derive(Debug, PartialEq, Clone)]
pub enum ParseNode {
    /// A simple symbol like 'x' or 'α'
    Symbol(Symbol),
    /// A group of nodes enclosed by '\left' and '\right'
    Delimited(Delimited),
    /// A group of nodes enclosed by a '\sqrt' square root radical.
    Radical(Radical),
    /// A fraction with some nodes as numerator and some other nodes in the denominator
    GenFraction(GenFraction),
    /// A node with superscripts or/and subscripts
    Scripts(Scripts),
    /// A rule (i.e. a uniformly filled line)
    Rule(Rule),
    /// Some (positive or negative) spacing between groups of nodes
    Kerning(AnyUnit),
    /// An accent over a certain groups of nodes
    Accent(Accent),
    /// A style (text cramped) to apply over a certain group of nodes
    Style(Style),
    /// A span of normal text without special math symbol replacement, spacing, etc.
    PlainText(PlainText),
    /// A change in the type of atoms
    AtomChange(AtomChange),
    /// A change in color
    Color(Color),
    /// A group of nodes
    Group(Vec<ParseNode>),
    /// Nodes stacked on top of each other with no alignment (the \substack command)
    Stack(Stack),
    /// Array of formulas, with some alignment
    Array(Array),


    // // DEPRECATED
    // /// Extend a glyph vertically ; this parse node is generated by the fictional \vextend LateX command.
    // /// It appears to be otherwise ignored
    // Extend(char, AnyUnit),
}


/// The collection of column formatting for an array.  This includes the vertical
/// alignment for each column in an array along with optional vertical bars
/// positioned to the right of the last column.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ArrayColumnsFormatting {
    /// The formatting specifications for each column
    pub columns: Vec<ArraySingleColumnFormatting>,

    /// The number of vertical marks after the last column.
    pub n_vertical_bars_before: u8,
}


/// Formatting options for a single column.  This includes both the horizontal
/// alignment of the column (clr), and optional vertical bar spacers (on the left).
#[derive(Default, Debug, Clone, Copy, PartialEq, Eq)]
pub struct ArraySingleColumnFormatting {
    /// The alignment of the column.  Defaults to Centered.
    pub alignment: ArrayColumnAlign,

    /// The number of vertical marks before column.
    pub n_vertical_bars_after: u8,
}

/// Array contents are the body of the enviornment.  Columns are seperated
/// by `&` and a newline is terminated by either:
///   - `\\[unit]`
///   - `\cr[unit]`
/// where a `[unit]` is any recognized dimension which will add (or subtract)
/// space between the rows.  Note, the last line termination is ignored
/// if the a line is empty.
pub type CellContent = Vec<ParseNode>;


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


/// An array of nodes as created by e.g. `\begin{array}{c} .. \end{array}` or `\begin{pmatrix} .. \end{pmatrix}`
#[derive(Debug, Clone, PartialEq)]
pub struct Array {
    /// The formatting arguments (clr) for each row.  Default: center.
    pub col_format: ArrayColumnsFormatting,

    /// A collection of rows.  Each row consists of one `Vec<Expression>`.
    pub rows: Vec<Vec<CellContent>>,

    /// The left delimiter for the array (optional).
    pub left_delimiter: Option<Symbol>,

    /// The right delimiter for the array (optional).
    pub right_delimiter: Option<Symbol>,
}


/// Cf [`ParseNode::Stack`]
#[derive(Debug, PartialEq, Clone)]
pub struct Stack {
    /// The type of the resulting stack.
    pub atom_type: AtomType,
    /// Lines of formulas to stack on top of each other.
    pub lines: Vec<Vec<ParseNode>>,
}

/// Cf [`ParseNode::Delimited`]
#[derive(Debug, PartialEq, Clone)]
pub struct Delimited {
    /// Symbols after \left, \middle and \right in the order that they appear
    delimiters : Vec<Symbol>,
    /// Nodes delimited by left, middle and right in the order that they appear
    inners:      Vec<Vec<ParseNode>>,
}

impl Delimited {
    /// Creates new [`Delimited`] from the given symbols and the given delimited group.
    pub fn new(delimiters: Vec<Symbol>, inners: Vec<Vec<ParseNode>>) -> Self 
    { Self { delimiters, inners } }

    /// Symbols after \left, \middle and \right in the order that they appear.
    pub fn delimiters(&self) -> &[Symbol] 
    { self.delimiters.as_ref() }

    /// Nodes delimited by left, middle and right in the order that they appear.
    pub fn inners(&self) -> &[Vec<ParseNode>] 
    { self.inners.as_ref() }
}

/// Cf [`ParseNode::Scripts`]
#[derive(Debug, PartialEq, Clone)]
pub struct Scripts {
    /// Nodes at the base.
    pub base: Option<Box<ParseNode>>,
    /// Superscripted nodes.
    pub superscript: Option<Vec<ParseNode>>,
    /// Subscripted nodes.
    pub subscript: Option<Vec<ParseNode>>,
}

impl Scripts {
    /// Retrieves the superscript if argument is true, otherwise the subscript. Useful for writing functions that work for both subscript and superscripts.
    pub fn get_script(&mut self, superscript : bool) -> &mut Option<Vec<ParseNode>> {
        if superscript {
            &mut self.superscript
        }
        else  {
            &mut self.subscript
        }
    }
}

/// Cf [`ParseNode::AtomChange`]
#[derive(Clone, Debug, PartialEq)]
pub struct AtomChange {
    /// New atom type
    pub at: AtomType,
    /// Inner nodes
    pub inner: Vec<ParseNode>,
}

/// Cf [`ParseNode::PlainText`]
#[derive(Clone, Debug, PartialEq)]
pub struct PlainText {
    /// Text to be renderered
    pub text: String,
}

/// Cf [`ParseNode::Accent`]
#[derive(Clone, Debug, PartialEq)]
pub struct Accent {
    /// The accent to place on top of the nodes.
    pub symbol: Symbol,
    /// The nodes "below" the accent.
    pub nucleus: Vec<ParseNode>,
}

/// Cf [`ParseNode::Rule`]. While intended to be used as lines, rules can in fact be any rectangle.
#[derive(Copy, Clone, PartialEq, Debug)]
pub struct Rule {
    /// width of the rule
    pub width: AnyUnit,
    /// height of the rule
    pub height: AnyUnit,
    //pub depth:  Unit,
}

/// Cf [`ParseNode::Radical`]
#[derive(Debug, PartialEq, Clone)]
pub struct Radical {
    /// The nodes that the root covers
    pub inner: Vec<ParseNode>,
    // pub superscript: Vec<ParseNode>,
}

/// Cf [`ParseNode::GenFraction`]
#[derive(Debug, PartialEq, Clone)]
pub struct GenFraction {
    /// nodes at the numerator.
    pub numerator: Vec<ParseNode>,
    /// nodes at the denominator.
    pub denominator: Vec<ParseNode>,
    /// thickness of the fraction line.
    pub bar_thickness: BarThickness,
    /// symbol opening the fraction.
    pub left_delimiter: Option<Symbol>,
    /// symbol closing the fraction.
    pub right_delimiter: Option<Symbol>,
    /// style for the whole fraction.
    pub style: MathStyle,
}

/// Cf [`ParseNode::Color`]
#[derive(Debug, Clone, PartialEq)]
pub struct Color {
    /// new color for the children nodes
    pub color: RGBA,
    /// children nodes
    pub inner: Vec<ParseNode>,
}

/// Type of thickness for fraction and binomials
#[derive(Debug, Copy, Clone, PartialEq)]
pub enum BarThickness {
    /// A default thickness
    Default,
    /// No bar
    None,
    /// A custom thickness
    Unit(AnyUnit),
}

/// Style of maths
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum MathStyle {
    /// default style ; characters are slanted.
    Display,
    /// text style ; characters are straight.
    Text,
    /// keep style inherited from higher nodes.
    NoChange,
}

impl ParseNode {

    /// if parse node is a single symbol, returns it. Otherwise, `None`.
    pub fn is_symbol(&self) -> Option<Symbol> {
        match *self {
            ParseNode::Symbol(sym) => Some(sym),
            ParseNode::Scripts(Scripts { ref base, .. }) =>
                base.as_ref().and_then(|b| b.is_symbol()),
            ParseNode::Accent(ref acc) => is_symbol(&acc.nucleus),
            ParseNode::AtomChange(ref ac) => is_symbol(&ac.inner),
            ParseNode::Color(ref clr) => is_symbol(&clr.inner),
            _ => None,
        }
    }

    /// sets atom type
    pub fn set_atom_type(&mut self, at: AtomType) {
        match *self {
            ParseNode::Symbol(ref mut sym) => sym.atom_type = at,
            ParseNode::Scripts(Scripts { ref mut base, .. }) => {
                if let Some(ref mut b) = *base {
                    b.set_atom_type(at);
                }
            }
            ParseNode::AtomChange(ref mut node) => node.at = at,
            ParseNode::Stack(Stack { ref mut atom_type, .. }) => *atom_type = at,
            _ => (),
        }
    }


    /// Get atom type of parse node
    pub fn atom_type(&self) -> AtomType {
        match *self {
            ParseNode::Symbol(ref sym)  => sym.atom_type,
            ParseNode::Delimited(_)     => AtomType::Inner,
            ParseNode::Radical(_)       => AtomType::Alpha,
            ParseNode::PlainText(_)     => AtomType::Alpha,
            ParseNode::GenFraction(_)   => AtomType::Inner,
            ParseNode::Group(_)         => AtomType::Alpha,
            ParseNode::Scripts(ref scr) => scr.base.as_ref()
                .map(|base| base.atom_type())
                .unwrap_or(AtomType::Alpha),

            ParseNode::Rule(_)          => AtomType::Alpha,
            ParseNode::Kerning(_)       => AtomType::Transparent,
            ParseNode::Accent(ref acc)  => acc.nucleus.first()
                .map(|acc| acc.atom_type())
                .unwrap_or(AtomType::Alpha),

            ParseNode::Style(_)         => AtomType::Transparent,
            ParseNode::AtomChange(ref ac) => ac.at,
            ParseNode::Color(ref clr)     => clr.inner.first()
                .map(|first| first.atom_type())
                .unwrap_or(AtomType::Alpha),

            ParseNode::Array(_)      => AtomType::Inner,
            ParseNode::Stack(ref s)  => s.atom_type,

            // // DEPRECATED
            // ParseNode::Extend(_,_)   => AtomType::Inner,
        }
    }
}

/// if `contents` is a singleton containing a symbol, returns the symbol ; otherwise, None.
pub fn is_symbol(contents: &[ParseNode]) -> Option<Symbol> {
    if contents.len() != 1 {
        return None;
    }

    contents[0].is_symbol()
}