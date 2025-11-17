use crate::parser::CommonErrors;
use crate::suggestion::Suggestion;
use crate::tokenizer::Tokenizer;
use crate::{parser::TsError, suggestion::Suggest};
use ariadne::{Color, Label, Report, ReportKind, Source};
use colored::*;

/// Pretty format
pub fn fmt(err: &TsError) -> String {
    let src = std::fs::read_to_string(&err.file).unwrap_or_default();
    if src.is_empty() {
        return fmt_simple(err);
    }

    let tokens = Tokenizer::new(src.clone()).tokenize();
    let mut span = 0..0;

    for token in &tokens {
        if token.line == err.line
            && (err.column - 1) >= token.column
            && (err.column - 1) < token.column + token.raw.chars().count()
        {
            span = token.start..token.end;
            break;
        }
    }

    let suggestion = Suggestion::build(err, &tokens);

    let mut buf = Vec::new();

    Report::build(ReportKind::Error, (&err.file, span.clone()))
        .with_code(&err.code)
        .with_message(&err.message)
        .with_label(
            Label::new((&err.file, span))
                .with_color(Color::Red)
                .with_message(
                    suggestion
                        .as_ref()
                        .and_then(|s| s.suggestion.clone())
                        .unwrap_or_else(|| "Error found here ".to_string()),
                ),
        )
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
