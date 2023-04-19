//! Parses strings representing LateX formulas into [`ParseNode`]'s
//! 
//! The main function function of interest is [`engine::parse`]

#[macro_use]
pub mod builders;
pub mod engine;
#[deny(missing_docs)]
pub mod nodes;
#[deny(missing_docs)]
pub mod color;
#[deny(missing_docs)]
pub mod symbols;

pub use self::engine::*;
pub use self::nodes::ParseNode;
pub use self::nodes::is_symbol;
