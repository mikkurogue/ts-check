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
    pub suggestions: Option<Vec<String>>,
    pub help: Option<String>,
}

impl Suggest for Suggestion {
    /// Build a suggestion and help text for the given TsError
    fn build(err: &TsError, tokens: &[Token]) -> Option<Self> {
        match err.code {
            CommonErrors::TypeMismatch => Some(Self {
                suggestion: type_mismatch_2322(err),
                suggestions: None,
                help: Some(
                    "Ensure that the types are compatible or perform an explicit conversion."
                        .to_string(),
                ),
            }),
            CommonErrors::InlineTypeMismatch => {
                let (suggestion, suggestions) = inline_type_mismatch_2345(err);
                Some(Self {
                    suggestion,
                    suggestions,
                    help: Some(
                        "Check the function arguments to ensure they match the expected parameter types."
                            .to_string(),
                    ),
                })
            }
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
                    suggestions: None,
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
                    suggestions: None,
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
                        suggestions: None,
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
                        suggestions: None,
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
                suggestions: None,
                help: Some("Review the comparison logic to ensure it makes sense.".to_string()),
            }),
            CommonErrors::Unsupported(_) => None,
        }
    }
}

/// Suggestion helper for ts2322
fn type_mismatch_2322(err: &TsError) -> Option<String> {
    if let Some((from, to)) = parse_ts2322_error(&err.message) {
        Some(format!(
            "Try converting this value from `{}` to `{}`.",
            from.red().bold(),
            to.green().bold()
        ))
    } else {
        None
    }
}

/// Suggestion helper for ts2345
fn inline_type_mismatch_2345(err: &TsError) -> (Option<String>, Option<Vec<String>>) {
    if let Some(mismatches) = parse_ts2345_error(&err.message) {
        if mismatches.is_empty() {
            return (None, None);
        }

        if mismatches.len() == 1 {
            let (property, provided, expected) = &mismatches[0];
            return (
                Some(format!(
                    "Property `{}` is provided as `{}` but expects `{}`.",
                    property.red().bold(),
                    provided.red().bold(),
                    expected.green().bold()
                )),
                None,
            );
        }

        let lines: Vec<String> = mismatches
            .iter()
            .map(|(property, provided, expected)| {
                format!(
                    "Property `{}` is provided as `{}` but expects `{}`.",
                    property.red().bold(),
                    provided.red().bold(),
                    expected.green().bold()
                )
            })
            .collect();

        (None, Some(lines))
    } else {
        (None, None)
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
                    for c2 in chars.by_ref() {
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
    if let Some(start_index) = msg.rfind(type_marker) {
        let rest_of_msg = &msg[start_index + type_marker.len()..];
        if let Some(end_index) = rest_of_msg.find('\'') {
            return Some(rest_of_msg[..end_index].to_string());
        }
    }
    None
}

fn parse_ts2345_error(msg: &str) -> Option<Vec<(String, String, String)>> {
    // Extract the provided and expected object types from the first line
    let provided_obj = extract_object_type(msg, "Argument of type '")?;
    let expected_obj = extract_object_type(msg, "to parameter of type '")?;

    // Parse both object types to extract their properties
    let provided_props = parse_object_properties(&provided_obj);
    let expected_props = parse_object_properties(&expected_obj);

    // Find all mismatched properties
    let mut mismatches = Vec::new();
    for (key, expected_type) in &expected_props {
        if let Some(provided_type) = provided_props.get(key)
            && provided_type != expected_type
        {
            mismatches.push((key.clone(), provided_type.clone(), expected_type.clone()));
        }
    }

    Some(mismatches)
}

fn extract_object_type(msg: &str, marker: &str) -> Option<String> {
    let start = msg.find(marker)? + marker.len();
    let rest = &msg[start..];
    let end = rest.find('\'')?;
    Some(rest[..end].to_string())
}

fn parse_object_properties(obj_type: &str) -> std::collections::HashMap<String, String> {
    let mut props = std::collections::HashMap::new();

    // Remove leading/trailing braces and whitespace
    let obj_type = obj_type.trim();
    if !obj_type.starts_with('{') || !obj_type.ends_with('}') {
        return props;
    }

    let inner = &obj_type[1..obj_type.len() - 1];

    // Split by semicolons to get individual properties
    for prop in inner.split(';') {
        let prop = prop.trim();
        if prop.is_empty() {
            continue;
        }

        // Split by colon to get key and type
        if let Some(colon_pos) = prop.find(':') {
            let key = prop[..colon_pos].trim().to_string();
            let value = prop[colon_pos + 1..].trim().to_string();
            props.insert(key, value);
        }
    }

    props
}
