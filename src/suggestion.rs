use crate::parser::{CommonErrors, TsError};
use crate::tokenizer::Token;
use colored::*;

pub trait Suggest {
    fn build(err: &TsError, tokens: &[Token]) -> Option<Self>
    where
        Self: Sized;
}

pub struct Suggestion {
    pub suggestion: Option<String>,
    pub help: Option<String>,
}

impl Suggest for Suggestion {
    fn build(err: &TsError, tokens: &[Token]) -> Option<Self> {
        let suggestion = match err.code {
            CommonErrors::TypeMismatch => Some(Self {
                suggestion: type_mismatch(err),
                help: Some(
                    "Ensure that the types are compatible or perform an explicit conversion."
                        .to_string(),
                ),
            }),
            CommonErrors::MissingParameters => {
                let mut fn_name = err
                    .message
                    .split('\'')
                    .nth(1)
                    .unwrap_or("function")
                    .to_string();

                for token in tokens {
                    if token.line == err.line
                        && (err.column - 1) >= token.column
                        && (err.column - 1) < token.column + token.raw.chars().count()
                    {
                        fn_name = token.raw.clone();
                        break;
                    }
                }

                Some(Self {
                    suggestion: Some(format!(
                        "Check if all required parameters are provided when invoking {}",
                        fn_name.red().bold()
                    )),
                    help: Some(format!(
                        "Function `{}` is missing 1 or more parameters.",
                        fn_name.red().bold()
                    )),
                })
            }
            CommonErrors::NoImplicitAny => {
                let param_name = err.message.split('\'').nth(1).unwrap_or("parameter");

                Some(Self {
                    suggestion: Some(format!("{} is implicitly `any`.", param_name.red().bold())),
                    help: Some(
                        "Consider adding type annotations to avoid implicit 'any' types."
                            .to_string(),
                    ),
                })
            }

            CommonErrors::PropertyMissingInType => {
                if let Some(type_name) = parse_property_missing_error(&err.message) {
                    let mut var_name: String = String::new();
                    for token in tokens {
                        if token.line == err.line
                            && (err.column - 1) >= token.column
                            && (err.column - 1) < token.column + token.raw.chars().count()
                        {
                            var_name = token.raw.clone();
                            break;
                        }
                    }

                    Some(Self {
                        suggestion: Some(format!(
                            "Verify that `{}` matches the annotated type `{}`.",
                            var_name.red().bold().italic(),
                            type_name.red().bold()
                        )),
                        help: Some(format!(
                            "Ensure that `{}` has all required properties defined in the type `{}`.",
                            var_name.red().bold().italic(),
                            type_name.red().bold()
                        )),
                    })
                } else {
                    Some(Self {
                        suggestion: Some(
                            "Verify that the object structure includes all required members of the specified type."
                                .to_string(),
                        ),
                        help: Some(
                            "Ensure the object has all required properties defined in the type."
                                .to_string(),
                        ),
                    })
                }
            }
            CommonErrors::UnintentionalComparison => Some(Self {
                suggestion: Some(
                    "Impossible to compare as left side value is narrowed to a single value."
                        .to_string(),
                ),
                help: Some("Review the comparison logic to ensure it makes sense.".to_string()),
            }),
            CommonErrors::Unsupported(_) => None,
        };

        suggestion
    }
}

/// Suggestion for the type mismatch error
fn type_mismatch(err: &TsError) -> Option<String> {
    if let Some((from, to)) = parse_ts2322_error(&err.message) {
        Some(format!(
            "Try converting this value from `{}` to `{}`.",
            from, to
        ))
    } else {
        None
    }
}

fn parse_ts2322_error(msg: &str) -> Option<(String, String)> {
    let mut chars = msg.chars().peekable();

    fn read_quoted<I: Iterator<Item = char>>(chars: &mut std::iter::Peekable<I>) -> Option<String> {
        // Expect starting `'`
        if chars.next()? != '\'' {
            return None;
        }
        let mut out = String::new();
        while let Some(&c) = chars.peek() {
            chars.next();
            if c == '\'' {
                break;
            }
            out.push(c);
        }
        Some(out)
    }

    while let Some(_) = chars.next() {
        let mut lookahead = chars.clone();
        if lookahead.next()? == 'y'
            && lookahead.next()? == 'p'
            && lookahead.next()? == 'e'
            && lookahead.next()? == ' '
            && lookahead.next()? == '\''
        {
            for _ in 0..4 {
                chars.next();
            }
            let from = read_quoted(&mut chars)?;

            while let Some(c) = chars.next() {
                if c == '\'' {
                    let mut secondary = String::new();
                    while let Some(c2) = chars.next() {
                        if c2 == '\'' {
                            break;
                        }
                        secondary.push(c2);
                    }
                    return Some((from, secondary));
                }
            }
        }
    }

    None
}

fn parse_property_missing_error(msg: &str) -> Option<String> {
    let type_marker = "type '";
    if let Some(start_index) = msg.find(type_marker) {
        let rest_of_msg = &msg[start_index + type_marker.len()..];
        if let Some(end_index) = rest_of_msg.find('\'') {
            return Some(rest_of_msg[..end_index].to_string());
        }
    }
    None
}
