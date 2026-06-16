use crate::{
    errors::error_renderer::ErrorRenderer, traits::frontend::Positional, types::frontend::Span,
};
use std::fmt;
pub mod error_renderer;

pub enum ErrorType {
    Error,
    Warning,
}

/// Simple parse error
pub struct SpannedError {
    message: String,
    tip: Option<String>,
    span: Span,

    error_type: ErrorType,
}

impl SpannedError {
    /// Create new parse error
    pub fn new(message: impl Into<String>, span: Span) -> Self {
        Self {
            message: message.into(),
            span,
            tip: None,
            error_type: ErrorType::Error,
        }
    }

    /// New error with tip
    pub fn new_tipped(message: impl Into<String>, tip: impl Into<String>, span: Span) -> Self {
        Self {
            message: message.into(),
            span,
            tip: Some(tip.into()),
            error_type: ErrorType::Error,
        }
    }

    /// Set error type
    pub fn set_type(mut self, t: ErrorType) -> Self {
        self.error_type = t;
        self
    }
}

impl Positional for SpannedError {
    fn get_span(&self) -> Span {
        self.span.clone()
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
        self.display(f)
    }
}

impl std::error::Error for SpannedError {}

/// Default unspanned error
#[derive(Default)]
pub struct CommonError {
    message: String,
}

impl CommonError {
    pub fn new(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
        }
    }
}

impl std::error::Error for CommonError {}

impl fmt::Display for CommonError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Error: {}", self.message)
    }
}

impl fmt::Debug for CommonError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("CommonError")
            .field("message", &self.message)
            .finish()
    }
}
