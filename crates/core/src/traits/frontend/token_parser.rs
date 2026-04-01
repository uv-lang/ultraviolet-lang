use crate::{errors::SpannedError, traits::frontend::Positional, types::frontend::Span};

pub trait UnwrapOptionError<T> {
    /// Unwrapping Option to a value or throw a spanned error
    fn unwrap_or_spanned(&self, parent_span: Span) -> Result<T, SpannedError>;
}

impl<T: Positional> Positional for &T {
    fn get_span(&self) -> Span {
        (*self).get_span()
    }
}
