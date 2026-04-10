pub mod ast;
pub mod token_parser;

use crate::types::frontend::Span;

/**
* Trait wrapper to get the span of the current block
*/
pub trait Positional {
    /// Get associated Span
    fn get_span(&self) -> Span;
}
