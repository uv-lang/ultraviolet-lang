pub mod ast;
pub mod token_parser;

use crate::types::frontend::Span;

pub trait Positional {
    /// Get associated Span
    fn get_span(&self) -> Span;
}
