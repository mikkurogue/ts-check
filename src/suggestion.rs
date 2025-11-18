use crate::parser::{CommonErrors, TsError};
use crate::tokenizer::Token;
use colored::*;

pub trait Suggest {
    fn build(err: &TsError, tokens: &[Token]) -> Option<Self>
    where
        Self: Sized;
}

pub struct Suggestion {
    pub suggestions: Vec<String>,
    pub help: Option<String>,
}

impl Suggest for Suggestion {
    /// Build a suggestion and help text for the given TsError
    fn build(err: &TsError, tokens: &[Token]) -> Option<Self> {
        match err.code {
            CommonErrors::TypeMismatch => Some(Self {
                suggestions: vec![type_mismatch_2322(err)?],
                help: Some(
                    "Ensure that the types are compatible or perform an explicit conversion."
                        .to_string(),
                ),
            }),
            CommonErrors::InlineTypeMismatch => {
                let suggestions = inline_type_mismatch_2345(err);
                Some(Self {
                    suggestions: suggestions.unwrap_or_default(),
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
                    suggestions: vec![format!(
                        "Check if all required arguments are provided when invoking {}",
                        fn_name.red().bold()
                    )],
                    help: Some(format!(
                        "Function `{}` is missing 1 or more arguments.",
                        fn_name.red().bold()
                    )),
                })
            }
            CommonErrors::NoImplicitAny => {
                let param_name = err.message.split('\'').nth(1).unwrap_or("parameter");

                Some(Self {
                    suggestions: vec![format!("{} is implicitly `any`.", param_name.red().bold())],
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
                        suggestions: vec![format!(
                            "Verify that `{}` matches the annotated type `{}`.",
                            var_name.red().bold().italic(),
                            type_name.red().bold()
                        )],
                        help: Some(format!(
                            "Ensure that `{}` has all required properties defined in the type `{}`.",
                            var_name.red().bold().italic(),
                            type_name.red().bold()
                        )),
                    })
                } else {
                    Some(Self {
                        suggestions: vec![
                            "Verify that the object structure includes all required members of the specified type."
                                .to_string()
                        ],
                        help: Some(
                            "Ensure the object has all required properties defined in the type."
                                .to_string(),
                        ),
                    })
                }
            }
            CommonErrors::UnintentionalComparison => Some(Self {
                suggestions: vec![
                    "Impossible to compare as left side value is narrowed to a single value."
                        .to_string()
                ],
                help: Some("Review the comparison logic to ensure it makes sense.".to_string()),
            }),
            CommonErrors::PropertyDoesNotExist => {
                let property_name = err.message.split('\'').nth(1).unwrap_or("property");
                let type_name = err.message.split('\'').nth(3).unwrap_or("type");

                Some(Self {
                    suggestions: vec![format!(
                        "Property `{}` does not exist on type `{}`.",
                        property_name.red().bold(),
                        type_name.red().bold()
                    )],
                    help: Some(format!(
                        "Add the missing property to the type `{}` or remove its usage.",
                        type_name.red().bold()
                    )),
                })
            }
            CommonErrors::ObjectIsPossiblyUndefined => {
                let possible_undefined_var = err
                    .message
                    .split('\'')
                    .nth(1)
                    .unwrap_or("object")
                    .to_string();

                Some(Self {
                    suggestions: vec![format!(
                        "{} may be `undefined` here.",
                        possible_undefined_var.red().bold()
                    )],
                    help: Some(format!(
                        "Consider optional chaining or an explicit check before attempting to access `{}`",
                        possible_undefined_var.red().bold()
                    )),
                })
            }
            CommonErrors::DirectCastPotentiallyMistaken => {
                let cast_from_type = err.message.split('\'').nth(1).unwrap_or("type");
                let cast_to_type = err.message.split('\'').nth(3).unwrap_or("type");

                Some(Self {
                    suggestions: vec![format!(
                        "Directly casting from `{}` to `{}` can be unsafe or mistaken, as both types do not overlap sufficiently.",
                        cast_from_type.yellow().bold(),
                        cast_to_type.yellow().bold()
                    )],
                    help: Some(format!(
                        "Consider using type guards or intermediate conversions to ensure type safety when casting from `{}` to `{}`, only intermediately cast `as unknown` if this is desired.",
                        cast_from_type.yellow().bold(),
                        cast_to_type.yellow().bold()
                    )),
                })
            },
            CommonErrors::SpreadArgumentMustBeTupleType => {
                Some(Self {
                    suggestions: vec![
                        "The argument being spread must be a tuple type or a `spreadable` type."
                            .to_string()
                    ],
                    help: Some(
                        "Ensure that the argument being spread is a tuple type compatible with the function's parameter type."
                            .to_string(),
                    ),
                })
            },
            CommonErrors::RightSideArithmeticMustBeEnumberable => Some(Self {
                suggestions: vec![
                    "The right-hand side of any arithmetic operation must be a number or enumerable."
                        .to_string()
                ],
                help: Some(
                    "Ensure that the value on the right side of the arithmetic operator is of type `number`, `bigint` or an enum member."
                        .to_string(),
                ),
            }),
            CommonErrors::LeftSideArithmeticMustBeEnumberable => Some(Self {
                suggestions: vec![
                    "The left-hand side of any arithmetic operation must be a number or enumerable."
                        .to_string()
                ],
                help: Some(
                    "Ensure that the value on the left side of the arithmetic operator is of type `number`, `bigint` or an enum member."
                        .to_string(),
                ),
            }),
            CommonErrors::IncompatibleOverload => Some(Self {
                suggestions: vec![
                    "The provided arguments do not match any overload of the function."
                        .to_string()
                ],
                help: Some(
                    "Check the function overloads and ensure that this signature adheres to the parent signature."
                        .to_string(),
                ),
            }),
            CommonErrors::InvalidShadowInScope => {
                let var_name = err.message.split('\'').nth(1).unwrap_or("variable");

                Some(Self {
                suggestions: vec![
                   format!("Declared variable `{}` can not shadow another variable in this scope.", var_name.red().bold()) 
                ],
                help: Some(
                        format!("Consider renaming the invalid shadowed variable `{}`.", var_name.red().bold()
),
                ),
            })
            },
            CommonErrors::NonExistentModuleImport => {
                let module_name = err.message.split('\'').nth(1).unwrap_or("module");

                Some(Self {
                    suggestions: vec![format!(
                        "Module `{}` does not exist.",
                        module_name.red().bold()
                    )],
                    help: Some(format!(
                        "Ensure that the module `{}` is installed and the import path is correct.",
                        module_name.red().bold(),
                    )),
                })
            }
            CommonErrors::ReadonlyPropertyAssignment => {
                let property_name = err.message.split('\'').nth(1).unwrap_or("property");

                Some(Self {
                    suggestions: vec![format!(
                        "Property `{}` is readonly and thus can not be re-assigned.",
                        property_name.red().bold()
                    )],
                    help: Some(format!(
                        "Consider removing the assignment to the read-only property `{}` or changing its declaration to be mutable.",
                        property_name.red().bold()
                    )),
                })
            }
            CommonErrors::IncorrectInterfaceImplementation => {
                let class_name = err.message.split('\'').nth(1).unwrap_or("class");
                let interface_name = err.message.split('\'').nth(3).unwrap_or("interface");

                // TODO: make a helper to extract all missing properties.
                let missing_property = err.message.split('\'').nth(5).unwrap_or("property");

                Some(Self {
                    suggestions: vec![format!(
                        "Class `{}` does not implement `{}` from interface `{}`.",
                        class_name.red().bold(),
                        missing_property.red().bold(),
                        interface_name.red().bold()
                    )],
                    help: Some(format!(
                        "Ensure that `{}` provides all required properties and methods defined in the interface `{}`.",
                        class_name.red().bold(),
                        interface_name.red().bold()
                    )),
                })
            }
            CommonErrors::PropertyInClassNotAssignableToBase => {
                let property = err.message.split('\'').nth(1).unwrap_or("property");
                let impl_type = err.message.split('\'').nth(3).unwrap_or("type");
                let base_type = err.message.split('\'').nth(5).unwrap_or("base type");
                
                let property_impl_type = err.message.split('\'').nth(7).unwrap_or("type");
                let property_base_type = err.message.split('\'').nth(9).unwrap_or("base type");

                Some(Self {
                    suggestions: vec![
                        format!(
                            "Property `{}` in class `{}` is not assignable to the same property in base class `{}`.",
                            property.red().bold(),
                            impl_type.red().bold(),
                            base_type.red().bold()
                        ),
                        format!(
                            "Property `{}` is implemented as type `{}` but defined as `{}`.",
                            property.red().bold(),
                            property_impl_type.red().bold(),
                            property_base_type.green().bold()
                        )

                    ],
                    help: Some(format!(
                        "Ensure that the type of property `{}` in class `{}` is compatible with the type defined in base class `{}`.",
                        property.red().bold(),
                        impl_type.red().bold(),
                        base_type.red().bold()
                    )),
                })
            }
            CommonErrors::CannotFindIdentifier => {
                let identifier = err.message.split('\'').nth(1).unwrap_or("identifier");

                Some(Self {
                    suggestions: vec![format!(
                        "Identifier `{}` cannot be found in the current scope.",
                        identifier.red().bold()
                    )],
                    help: Some(format!(
                        "Ensure that `{}` is declared and accessible in the current scope or remove this reference.",
                        identifier.red().bold()
                    )),
                })
            }
            CommonErrors::MissingReturnValue => {
                Some(Self {
                    suggestions: vec![
                        "A return value is missing where one is expected.".to_string()
                    ],
                    help: Some(
                        "A function that declares a return type must return a value of that type on all branches."
                            .to_string(),
                    ),
                })
            }
            CommonErrors::UncallableExpression => {
                let expr = err.message.split('\'').nth(1).unwrap_or("expression");

                Some(Self {
                    suggestions: vec![format!(
                        "Expression `{}` not can not be invoked or called.",
                        expr.red().bold()
                    )],
                    help: Some(format!(
                        "Ensure that `{}` is a function or has a callable signature before invoking it.",
                        expr.red().bold()
                    )),
                })
            }
            CommonErrors::InvalidIndexType => {
                let index_type = err.message.split('\'').nth(1).unwrap_or("type");

                Some(Self {
                    suggestions: vec![format!(
                        "`{}` cannot be used as an index accessor.",
                        index_type.red().bold()
                    )],
                    help: Some("Ensure that the index type is `number`, `string`, `symbole` or a compatible index type.".to_string(),
                    ),
                })
            }
            CommonErrors::TypoPropertyOnType => {
                let property_name = err.message.split('\'').nth(1).unwrap_or("property");
                let type_name = err.message.split('\'').nth(3).unwrap_or("type");
                let suggested_property_name = err.message.split('\'').nth(5).unwrap_or("property");

                Some(Self {
                    suggestions: vec![format!(
                        "Property `{}` does not exist on type `{}`. Try `{}` instead",
                        property_name.red().bold(),
                        type_name.yellow().bold(),
                        suggested_property_name.green().bold()
                    )],
                    help: Some(format!(
                        "Check for typos in the property name `{}` or ensure that it is defined on type `{}`.",
                        property_name.red().bold(),
                        type_name.red().bold()
                    )),
                })
            }

            CommonErrors::ObjectIsPossiblyNull => None,
            CommonErrors::ObjectIsUnknown => None,
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
fn inline_type_mismatch_2345(err: &TsError) -> Option<Vec<String>> {
    if let Some(mismatches) = parse_ts2345_error(&err.message) {
        if mismatches.is_empty() {
            return None;
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

        Some(lines)
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
