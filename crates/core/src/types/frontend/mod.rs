pub mod ast;
pub mod lexer;
pub mod number;
pub mod tokens;
pub mod typechecker;
pub mod types;

use core::fmt;
use std::{
    collections::HashMap,
    error::Error,
    fs,
    ops::{Deref, DerefMut},
    path::Path,
    rc::Rc,
};

use crate::{
    errors::CommonError,
    traits::frontend::{Positional, UVDisplay},
    types::frontend::ast::ASTBlockType,
};

/// Representation of input file
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct SourceFile {
    pub path: Box<Path>,
    pub code: String,

    /// Indexes of each line starts
    pub line_starts: Vec<usize>,

    /// char-byte mapping for proper slicing file
    char_to_byte: Vec<usize>,
}

impl SourceFile {
    /// Load source file from Path
    ///
    /// Returns `Err` when provided file not found or cannot be read
    pub fn load(path: &Path) -> Result<Self, CommonError> {
        let code: String = fs::read_to_string(path).map_err(|e| {
            CommonError::new(format!(
                "Cannot open source file `{}`: {}",
                path.to_string_lossy(),
                e
            ))
        })?;
        Ok(Self::from_str(code, path.into()))
    }

    /// Create source file from raw str
    pub fn from_str(str: impl Into<String>, path: Box<Path>) -> Self {
        let code: String = str.into();
        Self {
            path,
            code: code.clone(),
            char_to_byte: code.char_indices().map(|(i, _)| i).collect(),
            line_starts: std::iter::once(0)
                .chain(
                    code.chars()
                        .enumerate()
                        .filter(|(_, c)| *c == '\n')
                        .map(|(i, _)| i),
                )
                .collect(),
        }
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
    pub fn get_line_content(&self, line: usize) -> Result<&str, Box<dyn Error>> {
        let line_index_start = self.line_starts.get(line).ok_or(CommonError::default())?;
        let code_len = self.code.len();
        let line_index_end = self.line_starts.get(line + 1).unwrap_or(&code_len);

        // Convert char indexes to a bytes
        let line_start_byte = self
            .char_to_byte
            .get(*line_index_start)
            .ok_or(CommonError::default())?;
        let line_end_byte = self
            .char_to_byte
            .get(*line_index_end)
            .ok_or(CommonError::default())?;

        let line_content = self
            .code
            .get(*line_start_byte..*line_end_byte)
            .ok_or(CommonError::default())?
            .trim_end_matches("\n");

        Ok(line_content)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
/// Span displays the portion of the source code that a token or AST node occupies
pub struct Span {
    pub start: usize,
    pub end: usize,

    pub source_file: Rc<SourceFile>,
}

impl Span {
    /// Create new Span with provided start and end indexes
    pub fn new(s: usize, e: usize, sf: Rc<SourceFile>) -> Self {
        Self {
            start: s,
            end: e,
            source_file: sf,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
/// Type `T` with span
pub struct Spanned<T> {
    pub value: T,
    pub span: Span,
}

impl<T> Spanned<T> {
    pub fn new(value: T, span: Span) -> Self {
        Self { value, span }
    }

    /// Map contained value
    pub fn map<R, X: FnOnce(T) -> R>(self, f: X) -> Spanned<R> {
        Spanned::new(f(self.value), self.span)
    }

    /// Unwraps the inner value
    pub fn unwrap(self) -> T {
        self.value
    }
}

impl<T> Positional for Spanned<T> {
    fn get_span(&self) -> Span {
        self.span.clone()
    }
}

impl<T> Deref for Spanned<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.value
    }
}

impl<T> DerefMut for Spanned<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.value
    }
}

impl<T: fmt::Display> fmt::Display for Spanned<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.value)
    }
}

impl<T> UVDisplay for Vec<Spanned<T>>
where
    T: Into<String> + Clone,
{
    fn join(&self, del: &str) -> String {
        self.iter()
            .enumerate()
            .fold(String::default(), |mut acc, (i, el)| {
                acc.push_str(&el.value.clone().into());

                if i != self.len() - 1 {
                    acc.push_str(del);
                }
                acc
            })
    }
}

impl<T> Positional for Vec<Spanned<T>>
where
    T: Into<String> + Clone,
{
    fn get_span(&self) -> Span {
        let s = self.first().unwrap();
        let e = self.last().unwrap();

        Span::new(
            s.get_span().start,
            e.get_span().end,
            s.get_span().source_file,
        )
    }
}

pub struct SourceFileParsed {
    pub source: Rc<SourceFile>,
    pub ast: ASTBlockType,

    pub modules: HashMap<String, Rc<SourceFileParsed>>,
}
