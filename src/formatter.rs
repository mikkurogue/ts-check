use crate::error::{ErrorDiagnostic, TsError};
use crate::tokenizer::Tokenizer;
use ariadne::{Color, Label, Report, ReportKind, Source};
use colored::*;

/// Pretty format
pub fn fmt(err: &TsError) -> String {
    let src = std::fs::read_to_string(&err.file).unwrap_or_default();
    if src.is_empty() {
        return fmt_simple(err);
    }

    let tokens = Tokenizer::new(src.clone()).tokenize();
    let mut span = None;

    for token in &tokens {
        if token.line == err.line
            && (err.column - 1) >= token.column
            && (err.column - 1) < token.column + token.raw.chars().count()
        {
            span = Some(token.start..token.end);
            break;
        }
    }

    // If no token matched, calculate span from line/column
    let span = span.unwrap_or_else(|| {
        let mut byte_offset = 0;
        let mut current_line = 1;
        let mut current_column = 0;

        for ch in src.chars() {
            if current_line == err.line && current_column == err.column - 1 {
                // Found the position, use a small span for the character
                let char_len = ch.len_utf8();
                return byte_offset..byte_offset + char_len;
            }

            if ch == '\n' {
                current_line += 1;
                current_column = 0;
            } else {
                current_column += 1;
            }

            byte_offset += ch.len_utf8();
        }

        byte_offset.max(1) - 1..byte_offset
    });

    let suggestion = err.code.suggest(err, &tokens);

    let mut buf = Vec::new();

    // determine the span, either from tokens or the default
    let label_span = suggestion
        .as_ref()
        .and_then(|s| s.span.clone())
        .unwrap_or_else(|| span.clone());

    let mut report = Report::build(ReportKind::Error, (&err.file, span.clone()))
        .with_code(err.code)
        .with_message(&err.message);

    if let Some(ref s) = suggestion {
        if !s.suggestions.is_empty() {
            for suggestion_text in s.suggestions.iter() {
                report = report.with_label(
                    Label::new((&err.file, label_span.clone()))
                        .with_color(Color::Red)
                        .with_message(suggestion_text),
                );
            }
        } else {
            report = report.with_label(
                Label::new((&err.file, label_span.clone()))
                    .with_color(Color::Red)
                    .with_message("Error found here ".to_string()),
            );
        }
    } else {
        report = report.with_label(
            Label::new((&err.file, label_span))
                .with_color(Color::Red)
                .with_message("Error found here ".to_string()),
        );
    }

    report
        .with_help(
            suggestion
                .as_ref()
                .and_then(|s| s.help.clone())
                .unwrap_or_else(|| "No suggestion available.".to_string()),
        )
        .finish()
        .write((&err.file, Source::from(src)), &mut buf)
        .ok();

    String::from_utf8(buf).unwrap_or_else(|_| fmt_simple(err))
}

/// Simple formatting without src extraction
fn fmt_simple(err: &TsError) -> String {
    format!(
        "{}:{}:{} - {} {}: {}\n  --> {}:{}:{}\n      |\n      = TypeScript compiler error\n",
        err.file.cyan(),
        err.line.to_string().yellow(),
        err.column.to_string().yellow(),
        "error".red().bold(),
        err.code.to_string().red().bold(),
        err.message,
        err.file.cyan(),
        err.line.to_string().cyan(),
        err.column.to_string().cyan()
    )
}
