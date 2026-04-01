pub mod ast;
pub mod lexer;
pub mod tokens;

use anyhow::{Context, Result};
use std::{fs, ops::Deref, path::Path};

use crate::traits::frontend::Positional;

/// Representation of input file
pub struct SourceFile<'a> {
    pub path: &'a Path,
    pub code: String,

    /// Indexes of each line starts
    pub line_starts: Vec<usize>,

    /// char-byte mapping for proper slicing file
    char_to_byte: Vec<usize>,
}

impl<'a> SourceFile<'a> {
    /**
    Load source file from Path

    Returns `Err` when provided file not found or cannot be read
    */
    pub fn load(path: &'a Path) -> Result<Self> {
        let code: String = fs::read_to_string(path)?;
        Ok(Self {
            path,
            code: code.clone(),
            char_to_byte: code.char_indices().map(|(i, _)| i).collect(),
            line_starts: std::iter::once(0)
                .chain(code.chars().enumerate().filter(|(_, c)| *c == '\n').map(|(i, _)| i))
                .collect(),
        })
    }

    /// Get line and column of provided Span
    pub fn get_line_col(&self, span: Span) -> (usize, usize) {
        let line = self.get_line(span.start).unwrap_or(0);
        let column = span.start - self.line_starts[line];

        (line, column)
    }

    /// Search line No by provided Span start
    fn get_line(&self, target: usize) -> Option<usize> {
        if self.line_starts.is_empty() || target < self.line_starts[0] {
            return None;
        }

        match self.line_starts.binary_search(&target) {
            Ok(index) => Some(index),
            Err(index) => {
                if index > 0 {
                    Some(index - 1)
                } else {
                    None
                }
            },
        }
    }

    /// Get full line by provided line No
    pub fn get_line_content(&'a self, line: usize) -> Result<&'a str> {
        let line_index_start = self.line_starts.get(line).context("")?;
        let code_len = self.code.len();
        let line_index_end = self.line_starts.get(line + 1).unwrap_or(&code_len);

        // Convert char indexes to a bytes
        let line_start_byte = self.char_to_byte.get(*line_index_start).context("")?;
        let line_end_byte = self.char_to_byte.get(*line_index_end).context("")?;

        let line_content = self
            .code
            .get(*line_start_byte..*line_end_byte)
            .context("")?
            .trim_end_matches("\n");

        Ok(line_content)
    }
}

#[derive(Debug, Clone, PartialEq, Copy, Default)]
/// Span displays the portion of the source code that a token or AST node occupies
pub struct Span {
    pub start: usize,
    pub end: usize,
}

impl Span {
    /// Create new Span with provided start and end indexes
    pub fn new(s: usize, e: usize) -> Self {
        Self { start: s, end: e }
    }
}

#[derive(Debug, Clone, PartialEq)]
/// Type `T` with span
pub struct Spanned<T> {
    pub value: T,
    pub span: Span,
}

impl<T> Spanned<T> {
    pub fn new(value: T, span: Span) -> Self {
        Self { value, span }
    }
}

impl<T> Positional for Spanned<T> {
    fn get_span(&self) -> Span {
        self.span
    }
}

impl<T> Deref for Spanned<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.value
    }
}
