use crate::{
    errors::{CommonError, ErrorType, SpannedError},
    traits::frontend::Positional,
};
use colored::{Color, Colorize};
use std::{
    cmp::min,
    error::Error,
    fmt::{self, Write},
};

/// Trait for positional errors, that renders error messages
pub trait ErrorRenderer {
    /// Render error line syntax `<file>:<line>:<col>`
    fn render_error_line(&self, line: usize, col: usize) -> String;

    fn display(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result;

    /// Render extended error message
    fn render_extended(&self) -> Result<String, Box<dyn Error>>;
}

impl ErrorRenderer for SpannedError {
    fn display(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if let Ok(str) = self.render_extended() {
            return write!(f, "{str}");
        }

        let (line, col) = self.span.source_file.get_line_col(self.get_span());
        write!(
            f,
            "\n{}: {}",
            self.render_error_line(line, col).red(),
            self.message
        )
    }

    fn render_error_line(&self, line: usize, col: usize) -> String {
        format!(
            "{}:{}:{}",
            self.span.source_file.path.to_string_lossy(),
            line + 1,
            col
        )
    }

    fn render_extended(&self) -> Result<String, Box<dyn Error>> {
        let (line, col) = self.span.source_file.get_line_col(self.get_span());
        let mut line_content = self.span.source_file.get_line_content(line)?;
        let original_len = line_content.len();

        line_content = line_content.trim_start();
        let col_offsetted = col
            .checked_sub(original_len - line_content.len())
            .ok_or(CommonError::default())?;

        let error_line_link = self.render_error_line(line, col);

        let editor_line = line + 1;
        let line_no_len = editor_line.to_string().len();

        let mut output = String::new();

        let color = match self.error_type {
            ErrorType::Error => Color::Red,
            ErrorType::Warning => Color::Yellow,
        };

        let e_type = match self.error_type {
            ErrorType::Error => "error",
            ErrorType::Warning => "warning",
        };

        writeln!(output, "{}: {}", e_type.color(color), self.message.bold(),)?;
        if let Some(t) = &self.tip {
            writeln!(output, "{}: {}", "tip".green(), t.bold(),)?;
        }
        writeln!(output, " --> {error_line_link}")?;
        writeln!(output, " {} |", " ".repeat(line_no_len))?;
        writeln!(output, " {editor_line} | {line_content}")?;
        writeln!(
            output,
            " {} | {}{}",
            " ".repeat(line_no_len),
            " ".repeat(col_offsetted),
            "^".repeat(min(self.span.end - self.span.start, line_content.len() - 1))
                .color(color)
        )?;

        Ok(output)
    }
}
