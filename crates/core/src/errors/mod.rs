use std::fmt;

use crate::{traits::frontend::Positional, types::frontend::Span};

pub mod error_renderer;

/// Simple parse error
pub struct SpannedError {
    message: String,
    tip: Option<String>,
    span: Span,
}

impl SpannedError {
    /// Create new parse error
    pub fn new(message: impl Into<String>, span: Span) -> Self {
        Self {
            message: message.into(),
            span,
            tip: None,
        }
    }

    pub fn new_tipped(message: impl Into<String>, tip: impl Into<String>, span: Span) -> Self {
        Self {
            message: message.into(),
            span,
            tip: Some(tip.into()),
        }
    }
}

impl Positional for SpannedError {
    fn get_span(&self) -> Span {
        self.span
    }
}

impl fmt::Debug for SpannedError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("SpannedError")
            .field("message", &self.message)
            .field("span", &self.span)
            .finish()
    }
}

impl fmt::Display for SpannedError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("SpannedError")
            .field("message", &self.message)
            .field("span", &self.span)
            .finish()
    }
}

impl std::error::Error for SpannedError {}
