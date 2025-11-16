use crate::parser::CommonErrors;
use crate::suggestion::Suggestion;
use crate::{parser::TsError, suggestion::Suggest};
use ariadne::{Color, Label, Report, ReportKind, Source};
use colored::*;

/// Pretty format
pub fn fmt(err: &TsError) -> String {
    let src = std::fs::read_to_string(&err.file).unwrap_or_default();
    if src.is_empty() {
        return fmt_simple(err);
    }
    let span = tokenize_name(err, &src);

    let suggestion = Suggestion::build(err);

    let mut buf = Vec::new();
    Report::build(ReportKind::Error, (&err.file, span.clone()))
        .with_code(&err.code)
        .with_message(&err.message)
        .with_label(
            Label::new((&err.file, span))
                .with_color(Color::Red)
                .with_message(suggestion.unwrap_or_else(|| "Error found here ".to_string())),
        )
        .finish()
        .write((&err.file, Source::from(src)), &mut buf)
        .ok();
    String::from_utf8(buf).unwrap_or_else(|_| fmt_simple(err))
}

/// Tokenize the variable name where we found the error so its easier to highlight
fn tokenize_name(err: &TsError, src: &str) -> std::ops::Range<usize> {
    let mut offset = 0usize; // byte offset of start of current line
    for (i, line) in src.lines().enumerate() {
        let line_number = i + 1;
        if line_number == err.line {
            if line.is_empty() {
                return 0..0;
            }
            let col0 = err.column.saturating_sub(1).min(line.len());
            let bytes = line.as_bytes();
            let is_ident_char = |c: u8| c.is_ascii_alphanumeric() || c == b'_';
            let mut start_col = col0;
            let mut end_col = col0 + 1;
            if bytes[col0] == b'"' {
                // string literal
                end_col = col0 + 1;
                while end_col < bytes.len() && bytes[end_col] != b'"' {
                    end_col += 1;
                }
                if end_col < bytes.len() {
                    end_col += 1;
                }
            } else if is_ident_char(bytes[col0]) {
                while start_col > 0 && is_ident_char(bytes[start_col - 1]) {
                    start_col -= 1;
                }
                while end_col < bytes.len() && is_ident_char(bytes[end_col]) {
                    end_col += 1;
                }
            }
            let start = offset + start_col;
            let end = (offset + end_col).min(src.len());
            return start..end;
        }
        offset += line.len() + 1; // account for '\n'
    }
    0..0
}

/// Simple formatting without src extraction
pub fn fmt_simple(err: &TsError) -> String {
    let mut out = String::new();
    let code_str = CommonErrors::from_code(&err.code.to_string());

    out.push_str(&format!(
        "{}:{}:{} - {} {}: {}\n",
        err.file.cyan(),
        err.line.to_string().yellow(),
        err.column.to_string().yellow(),
        "error".red().bold(),
        code_str.to_string().red().bold(),
        err.message
    ));

    out.push_str(&format!(
        "  --> {}:{}:{}\n",
        err.file.cyan(),
        err.line.to_string().cyan(),
        err.column.to_string().cyan()
    ));

    // MVP: We do not extract source code yet — that’s a next step.
    out.push_str("      |\n");
    out.push_str(&format!(
        "{} {}\n",
        "      =".purple(),
        "TypeScript compiler error"
    ));

    out
}
