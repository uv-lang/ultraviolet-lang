use anyhow::{Context, Result};
use colored::Colorize;
use std::{cmp::min, fmt::Write};

use crate::{errors::SpannedError, traits::frontend::Positional, types::frontend::SourceFile};

/// Trait for positional errors, that renders error messages
pub trait ErrorRenderer {
    /// Render error line syntax `<file>:<line>:<col>`
    fn render_error_line(&self, line: usize, col: usize, source: &SourceFile) -> String;

    /// Display simple error with message
    fn display_with_source(&self, source: &SourceFile) -> String;

    /// Render extended error message
    fn render_extended(&self, source: &SourceFile) -> Result<String>;
}

impl ErrorRenderer for SpannedError {
    fn render_error_line(&self, line: usize, col: usize, source: &SourceFile) -> String {
        format!("{}:{}:{}", source.path.to_string_lossy(), line + 1, col)
    }

    fn display_with_source(&self, source: &SourceFile) -> String {
        if let Ok(str) = self.render_extended(source) {
            return str;
        }

        let (line, col) = source.get_line_col(self.get_span());
        format!(
            "\n{}: {}",
            self.render_error_line(line, col, source).red(),
            self.message
        )
    }

    fn render_extended(&self, source: &SourceFile) -> Result<String> {
        let (line, col) = source.get_line_col(self.get_span());
        let mut line_content = source.get_line_content(line)?;
        let original_len = line_content.len();

        line_content = line_content.trim_start();
        let col_offsetted = col.checked_sub(original_len - line_content.len()).context("")?;

        let error_line_link = self.render_error_line(line, col, source);

        let editor_line = line + 1;
        let line_no_len = editor_line.to_string().len();

        let mut output = String::new();
        writeln!(output, "{}: {}", "error".red(), self.message.bold())?;
        if let Some(t) = &self.tip {
            writeln!(output, "{}: {}", "tip".green(), t.bold())?;
        }
        writeln!(output, " --> {}", error_line_link)?;
        writeln!(output, " {} |", " ".repeat(line_no_len))?;
        writeln!(output, " {} | {}", editor_line, line_content)?;
        writeln!(
            output,
            " {} | {}{}",
            " ".repeat(line_no_len),
            " ".repeat(col_offsetted),
            "^".repeat(min(self.span.end - self.span.start, line_content.len() - 1))
                .red()
        )?;

        Ok(output)
    }
}
