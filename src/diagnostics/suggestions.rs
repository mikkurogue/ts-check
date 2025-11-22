use colored::*;

use crate::{
    error::{
        codes::ErrorCode,
        core::TsError,
        diagnostics::ErrorDiagnostic,
    },
    message_parser::{
        extract_first_quoted,
        extract_quoted_value,
        extract_second_quoted,
        extract_third_quoted,
        parse_property_missing_error,
        parse_ts2322_error,
        parse_ts2345_error,
    },
    suggestion::Suggestion,
    token_utils::{
        extract_function_name,
        extract_identifier_at_error,
        extract_identifier_or_default,
        find_identifier_after_keyword,
        find_token_at_position,
    },
    tokenizer::Token,
};

impl ErrorDiagnostic for ErrorCode {
    fn suggest(&self, err: &TsError, tokens: &[Token]) -> Option<Suggestion> {
        match self {
            ErrorCode::TypeMismatch => suggest_type_mismatch(err, tokens),
            ErrorCode::InlineTypeMismatch => suggest_inline_type_mismatch(err),
            ErrorCode::MissingParameters => suggest_missing_parameters(err, tokens),
            ErrorCode::NoImplicitAny => suggest_no_implicit_any(err),
            ErrorCode::PropertyMissingInType => suggest_property_missing_in_type(err, tokens),
            ErrorCode::UnintentionalComparison => suggest_unintentional_comparison(),
            ErrorCode::PropertyDoesNotExist => suggest_property_does_not_exist(err),
            ErrorCode::ObjectIsPossiblyUndefined => suggest_possibly_undefined(err),
            ErrorCode::DirectCastPotentiallyMistaken => suggest_direct_cast_mistaken(err),
            ErrorCode::SpreadArgumentMustBeTupleType => suggest_spread_tuple(err),
            ErrorCode::RightSideArithmeticMustBeEnumberable => suggest_right_arithmetic(err),
            ErrorCode::LeftSideArithmeticMustBeEnumberable => suggest_left_arithmetic(err),
            ErrorCode::IncompatibleOverload => suggest_incompatible_overload(err),
            ErrorCode::InvalidShadowInScope => suggest_invalid_shadow(err),
            ErrorCode::NonExistentModuleImport => suggest_nonexistent_module(err),
            ErrorCode::ReadonlyPropertyAssignment => suggest_readonly_property(err),
            ErrorCode::IncorrectInterfaceImplementation => suggest_incorrect_interface(err),
            ErrorCode::PropertyInClassNotAssignableToBase => suggest_property_not_assignable(err),
            ErrorCode::CannotFindIdentifier => suggest_cannot_find_identifier(err),
            ErrorCode::MissingReturnValue => suggest_missing_return(err),
            ErrorCode::UncallableExpression => suggest_uncallable_expression(err),
            ErrorCode::InvalidIndexType => suggest_invalid_index_type(err),
            ErrorCode::InvalidIndexTypeSignature => suggest_invalid_index_signature(err, tokens),
            ErrorCode::TypoPropertyOnType => suggest_typo_property(err),
            ErrorCode::ObjectIsPossiblyNull => suggest_possibly_null(err),
            ErrorCode::ObjectIsUnknown => suggest_object_unknown(err),
            ErrorCode::UnterminatedStringLiteral => suggest_unterminated_string(err),
            ErrorCode::IdentifierExpected => suggest_identifier_expected(),
            ErrorCode::DisallowedTrailingComma => suggest_disallowed_comma(),
            ErrorCode::SpreadParameterMustBeLast => suggest_spread_parameter_last(),
            ErrorCode::ExpressionExpected => suggest_expression_expected(),
            ErrorCode::UniqueObjectMemberNames => suggest_unique_members(),
            ErrorCode::UninitializedConst => suggest_uninitialized_const(err, tokens),
            ErrorCode::YieldNotInGenerator => suggest_yield_not_in_generator(),
            ErrorCode::JsxFlagNotProvided => suggest_jsx_flag(),
            ErrorCode::DeclaredButNeverUsed => suggest_declared_unused(err),
            ErrorCode::NoExportedMember => suggest_no_exported_member(err),
            ErrorCode::ImportedButNeverUsed => suggest_imported_unused(),
            ErrorCode::InvalidDefaultImport => suggest_invalid_default_import(),
            ErrorCode::UnreachableCode => suggest_unreachable(),
            ErrorCode::TypeAssertionInJsNotAllowed => suggest_type_assertion_in_js_not_allowed(),
            ErrorCode::MappedTypeMustBeStatic => suggest_mapped_type_must_be_static(),
            ErrorCode::ElementImplicitAnyInvalidIndexTypeForObject => {
                suggest_element_implicit_any_invalid_index_type_for_object(err)
            }
            ErrorCode::MissingJsxIntrinsicElementsDeclaration => {
                suggest_missing_jsx_intrinsic_elements_declaration()
            }
            ErrorCode::ConstEnumsDisallowed => suggest_const_enums_disallowed(),
            ErrorCode::JsxModuleNotSet => suggest_jsx_not_set(err),
            ErrorCode::UnexpectedKeywordOrIdentifier => {
                suggest_unexpected_kw_or_identifier(err, tokens)
            }
            ErrorCode::Unsupported(_) => None,
        }
    }
}

/// Suggestion for unexpected keyword or identifier
fn suggest_unexpected_kw_or_identifier(err: &TsError, tokens: &[Token]) -> Option<Suggestion> {
    let keyword = find_token_at_position(tokens, err.line, err.column)?;

    Some(Suggestion {
        suggestions: vec![format!(
            "{} `{}` is not expected in this context.",
            "[FATAL]".bright_red().bold().italic(),
            keyword.raw.bright_yellow().bold()
        )],
        help:        Some(
            "Avoid using unknown, undeclared or invalid keywords or identifiers.".to_string(),
        ),
        span:        None,
    })
}

/// Suggestion for when jsx compiler flag is not set but jsx is used
fn suggest_jsx_not_set(err: &TsError) -> Option<Suggestion> {
    let module_name = extract_first_quoted(&err.message)?;
    let resolved_name = extract_second_quoted(&err.message)?;

    Some(Suggestion {
        suggestions: vec![format!(
            "Module `{}` is resolved to `{}` but jsx compiler flag is not set.",
            module_name.red().bold(),
            resolved_name.red().bold()
        )],
        help:        Some("Enable `--jsx` compiler flag or add jsx to tsconfig.json".to_string()),
        span:        None,
    })
}

/// Suggestion for when isolatedModules is enabled and ambiend const enums are used.
fn suggest_const_enums_disallowed() -> Option<Suggestion> {
    Some(Suggestion {
        suggestions: vec![
            "Disable `isolatedModules` as a compiler setting to allow const enums.".to_string(),
        ],
        help:        Some(
            "Const enums are not valid when `isolatedModules` is enabled.".to_string(),
        ),
        span:        None,
    })
}

/// Suggestion for missing JSX intrinsic elements declaration
/// explained here https://www.totaltypescript.com/what-is-jsx-intrinsicelements
fn suggest_missing_jsx_intrinsic_elements_declaration() -> Option<Suggestion> {
    Some(Suggestion {
        suggestions: vec![
            "JSX intrinsic elements declaration is missing in global scope.".to_string(),
        ],
        help:        Some(
            "Either declare a global module with a JSX namespace or configure React or other JSX consumers correctly"
                .to_string(),
        ),
        span:        None,
    })
}

/// Suggestion for when element is implicitly any and that index type is invalid for indexing
/// object
fn suggest_element_implicit_any_invalid_index_type_for_object(err: &TsError) -> Option<Suggestion> {
    let implicit_type = extract_quoted_value(&err.message, 1)?;

    let index_type = extract_quoted_value(&err.message, 3)?;
    let object_to_index = extract_quoted_value(&err.message, 6)?;

    Some(Suggestion {
        suggestions: vec![format!(
            "`{}` can not be used as an index to access `{}` - therefore element is implicitly `{}`.",
            index_type.red().bold(),
            object_to_index.red().bold(),
            implicit_type.red().bold()
        )],
        help:        Some(format!(
            "Consider declaring the index with `{}` or loosen the type of `{}` to allow indexing with `{}`.",
            format!(
                "{} {}",
                "keyof typeof".yellow().bold(),
                object_to_index.yellow().bold()
            ),
            object_to_index.yellow().bold(),
            index_type.yellow().bold()
        )),
        span:        None,
    })
}

/// Suggestion for mapped types with non-static keys
fn suggest_mapped_type_must_be_static() -> Option<Suggestion> {
    Some(Suggestion {
        suggestions: vec![
            "Consider removing the properties and/or methods".to_string(),
        ],
        help:        Some(
            "Split multiple mapped property declarations into individual types and combine them using a type intersection."
                .to_string(),
        ),
        span:        None,
    })
}

/// Suggestiong for using type assertions and annotations outside of TypeScript files
fn suggest_type_assertion_in_js_not_allowed() -> Option<Suggestion> {
    Some(Suggestion {
        suggestions: vec!["Type assertions are not allowed in JavaScript files.".to_string()],
        help:        Some(
            "Consider converting the file to TypeScript or removing the type assertion."
                .to_string(),
        ),
        span:        None,
    })
}

/// Suggestion for TS95050
fn suggest_unreachable() -> Option<Suggestion> {
    Some(Suggestion {
        suggestions: vec!["Code here is unreachable".to_string()],
        help:        Some("Consider removing unreachable code or the statement that causes this to be unreachable".to_string()),
        span:        None,
    })
}

// Suggestion functions
fn suggest_type_mismatch(err: &TsError, tokens: &[Token]) -> Option<Suggestion> {
    if let Some((from, to)) = parse_ts2322_error(&err.message) {
        let var_name = extract_identifier_or_default(err, tokens, "");

        Some(Suggestion {
            suggestions: vec![format!(
                "Try converting `{}` from `{}` to `{}`.",
                var_name.yellow().bold().italic(),
                from.red().bold(),
                to.green().bold()
            )],
            help:        Some(
                "Ensure that the types are compatible or perform an explicit conversion."
                    .to_string(),
            ),
            span:        None,
        })
    } else {
        None
    }
}

fn suggest_inline_type_mismatch(err: &TsError) -> Option<Suggestion> {
    if err
        .message
        .contains("Target signature provides too few arguments")
    {
        let (expected, got) = if let Some(expected_str) = err.message.split("Expected ").nth(1) {
            let expected_num = expected_str
                .split(" or more")
                .next()
                .and_then(|s| s.trim().parse::<u32>().ok())
                .unwrap_or(0);
            let got_num = expected_str
                .split("but got ")
                .nth(1)
                .and_then(|s| s.split('.').next())
                .and_then(|s| s.trim().parse::<u32>().ok())
                .unwrap_or(0);
            (expected_num, got_num)
        } else {
            (0, 0)
        };

        let suggestion = if expected > 0 && got > 0 {
            format!(
                "The callback function has {} parameters, but the signature only accepts {}.",
                expected, got
            )
        } else {
            "The callback function has too many parameters for the expected signature.".to_string()
        };

        return Some(Suggestion {
            suggestions: vec![suggestion],
            help: Some(
                "Remove the extra parameters from the callback function to match the expected signature.".to_string()
            ),
            span: None,
        });
    }

    if err
        .message
        .contains("Target signature provides too many arguments")
    {
        return Some(Suggestion {
            suggestions: vec![
                "The callback function has too few parameters for the expected signature."
                    .to_string(),
            ],
            help: Some(
                "Add the missing parameters to the callback function to match the expected signature.".to_string()
            ),
            span: None,
        });
    }

    let suggestions = if let Some(mismatches) = parse_ts2345_error(&err.message) {
        if mismatches.is_empty() {
            None
        } else {
            Some(
                mismatches
                    .iter()
                    .map(|(property, provided, expected)| {
                        format!(
                            "Property `{}` is provided as `{}` but expects `{}`.",
                            property.red().bold(),
                            provided.red().bold(),
                            expected.green().bold()
                        )
                    })
                    .collect(),
            )
        }
    } else {
        None
    };

    Some(Suggestion {
        suggestions: suggestions.unwrap_or_else(|| {
            vec!["Argument type does not match the expected parameter type.".to_string()]
        }),
        help:        Some(
            "Check the function arguments to ensure they match the expected parameter types."
                .to_string(),
        ),
        span:        None,
    })
}

fn suggest_missing_parameters(err: &TsError, tokens: &[Token]) -> Option<Suggestion> {
    let fn_name = if let Some(name) = extract_identifier_at_error(err, tokens) {
        if !name.is_empty() {
            name
        } else {
            let fallback =
                extract_first_quoted(&err.message).unwrap_or_else(|| "function".to_string());
            extract_function_name(err, tokens, &fallback)
        }
    } else {
        let fallback = extract_first_quoted(&err.message).unwrap_or_else(|| "function".to_string());
        extract_function_name(err, tokens, &fallback)
    };

    let (expected, got) = if let Some(expected_str) = err.message.split("Expected ").nth(1) {
        let expected_num = expected_str
            .split(" argument")
            .next()
            .and_then(|s| s.trim().parse::<u32>().ok());
        let got_num = expected_str
            .split("but got ")
            .nth(1)
            .and_then(|s| s.split('.').next())
            .and_then(|s| s.trim().parse::<u32>().ok());
        (expected_num, got_num)
    } else {
        (None, None)
    };

    let (suggestion, help) = match (expected, got) {
        (Some(exp), Some(g)) if g < exp => (
            format!(
                "Function `{}` expects {} arguments but only received {}.",
                fn_name.red().bold(),
                exp,
                g
            ),
            format!(
                "Add the missing {} to match the expected signature.",
                if exp - g == 1 {
                    "argument"
                } else {
                    "arguments"
                }
            ),
        ),
        (Some(exp), Some(g)) if g > exp => (
            format!(
                "Function `{}` expects {} arguments but received {}.",
                fn_name.red().bold(),
                exp,
                g
            ),
            format!(
                "Remove the extra {} to match the expected signature.",
                if g - exp == 1 {
                    "argument"
                } else {
                    "arguments"
                }
            ),
        ),
        _ => (
            format!(
                "Check if all required arguments are provided when invoking {}",
                fn_name.red().bold()
            ),
            format!(
                "Ensure the correct number of arguments are passed to `{}`.",
                fn_name.red().bold()
            ),
        ),
    };

    Some(Suggestion {
        suggestions: vec![suggestion],
        help:        Some(help),
        span:        None,
    })
}

fn suggest_no_implicit_any(err: &TsError) -> Option<Suggestion> {
    let param_name = extract_first_quoted(&err.message)?;

    Some(Suggestion {
        suggestions: vec![format!("{} is implicitly `any`.", param_name.red().bold())],
        help:        Some(
            "Consider adding type annotations to avoid implicit 'any' types.".to_string(),
        ),
        span:        None,
    })
}

fn suggest_property_missing_in_type(err: &TsError, tokens: &[Token]) -> Option<Suggestion> {
    if let Some(type_name) = parse_property_missing_error(&err.message) {
        let var_name = extract_identifier_or_default(err, tokens, "");

        Some(Suggestion {
            suggestions: vec![format!(
                "Verify that `{}` matches the annotated type `{}`.",
                var_name.red().bold().italic(),
                type_name.red().bold()
            )],
            help:        Some(format!(
                "Ensure that `{}` has all required properties defined in the type `{}`.",
                var_name.red().bold().italic(),
                type_name.red().bold()
            )),
            span:        None,
        })
    } else {
        Some(Suggestion {
            suggestions: vec![
                "Verify that the object structure includes all required members of the specified type."
                    .to_string()
            ],
            help: Some(
                "Ensure the object has all required properties defined in the type."
                    .to_string(),
            ),
            span: None,
        })
    }
}

fn suggest_unintentional_comparison() -> Option<Suggestion> {
    Some(Suggestion {
        suggestions: vec![
            "Impossible to compare as left side value is narrowed to a single value.".to_string(),
        ],
        help:        Some("Review the comparison logic to ensure it makes sense.".to_string()),
        span:        None,
    })
}

fn suggest_property_does_not_exist(err: &TsError) -> Option<Suggestion> {
    let property_name = extract_first_quoted(&err.message)?;
    let type_name = extract_second_quoted(&err.message)?;

    Some(Suggestion {
        suggestions: vec![format!(
            "Property `{}` is not found on type `{}`.",
            property_name.red().bold(),
            type_name.red().bold()
        )],
        help:        Some(
            "Ensure the property exists on the type or adjust your code to avoid accessing it."
                .to_string(),
        ),
        span:        None,
    })
}

fn suggest_possibly_undefined(err: &TsError) -> Option<Suggestion> {
    let possible_undefined_var = extract_first_quoted(&err.message)?;

    Some(Suggestion {
        suggestions: vec![format!(
            "{} may be `undefined` here.",
            possible_undefined_var.red().bold()
        )],
        help:        Some(format!(
            "Consider optional chaining or an explicit check before attempting to access `{}`",
            possible_undefined_var.red().bold()
        )),
        span:        None,
    })
}

fn suggest_direct_cast_mistaken(err: &TsError) -> Option<Suggestion> {
    let cast_from_type = extract_first_quoted(&err.message)?;
    let cast_to_type = extract_second_quoted(&err.message)?;

    Some(Suggestion {
        suggestions: vec![format!(
            "Directly casting from `{}` to `{}` can be unsafe or mistaken, as both types do not overlap sufficiently.",
            cast_from_type.yellow().bold(),
            cast_to_type.yellow().bold()
        )],
        help:        Some(format!(
            "Consider using type guards or intermediate conversions to ensure type safety when casting from `{}` to `{}`, only intermediately cast `as unknown` if this is desired.",
            cast_from_type.yellow().bold(),
            cast_to_type.yellow().bold()
        )),
        span:        None,
    })
}

fn suggest_spread_tuple(_err: &TsError) -> Option<Suggestion> {
    Some(Suggestion {
        suggestions: vec![
            "The argument being spread must be a tuple type or a `spreadable` type.".to_string(),
        ],
        help: Some(
            "Ensure that the argument being spread is a tuple type compatible with the function's parameter type."
                .to_string(),
        ),
        span: None,
    })
}

fn suggest_right_arithmetic(_err: &TsError) -> Option<Suggestion> {
    Some(Suggestion {
        suggestions: vec![
            "The right-hand side of any arithmetic operation must be a number or enumerable."
                .to_string(),
        ],
        help: Some(
            "Ensure that the value on the right side of the arithmetic operator is of type `number`, `bigint` or an enum member."
                .to_string(),
        ),
        span: None,
    })
}

fn suggest_left_arithmetic(_err: &TsError) -> Option<Suggestion> {
    Some(Suggestion {
        suggestions: vec![
            "The left-hand side of any arithmetic operation must be a number or enumerable."
                .to_string(),
        ],
        help: Some(
            "Ensure that the value on the left side of the arithmetic operator is of type `number`, `bigint` or an enum member."
                .to_string(),
        ),
        span: None,
    })
}

fn suggest_incompatible_overload(_err: &TsError) -> Option<Suggestion> {
    Some(Suggestion {
        suggestions: vec![
            "The provided arguments do not match any overload of the function.".to_string(),
        ],
        help: Some(
            "Check the function overloads and ensure that this signature adheres to the parent signature."
                .to_string(),
        ),
        span: None,
    })
}

fn suggest_invalid_shadow(err: &TsError) -> Option<Suggestion> {
    let var_name = extract_first_quoted(&err.message)?;

    Some(Suggestion {
        suggestions: vec![format!(
            "Declared variable `{}` can not shadow another variable in this scope.",
            var_name.red().bold()
        )],
        help:        Some(format!(
            "Consider renaming the invalid shadowed variable `{}`.",
            var_name.red().bold()
        )),
        span:        None,
    })
}

fn suggest_nonexistent_module(err: &TsError) -> Option<Suggestion> {
    let module_name = extract_first_quoted(&err.message)?;

    Some(Suggestion {
        suggestions: vec![format!(
            "Module `{}` does not exist.",
            module_name.red().bold()
        )],
        help:        Some(format!(
            "Ensure that the module `{}` is installed and the import path is correct.",
            module_name.red().bold(),
        )),
        span:        None,
    })
}

fn suggest_readonly_property(err: &TsError) -> Option<Suggestion> {
    let property_name = extract_first_quoted(&err.message)?;

    Some(Suggestion {
        suggestions: vec![format!(
            "Property `{}` is readonly and thus can not be re-assigned.",
            property_name.red().bold()
        )],
        help:        Some(format!(
            "Consider removing the assignment to the read-only property `{}` or changing its declaration to be mutable.",
            property_name.red().bold()
        )),
        span:        None,
    })
}

fn suggest_incorrect_interface(err: &TsError) -> Option<Suggestion> {
    let class_name = extract_first_quoted(&err.message)?;
    let interface_name = extract_second_quoted(&err.message)?;
    let missing_property = extract_third_quoted(&err.message)?;

    Some(Suggestion {
        suggestions: vec![format!(
            "Class `{}` does not implement `{}` from interface `{}`.",
            class_name.red().bold(),
            missing_property.red().bold(),
            interface_name.red().bold()
        )],
        help:        Some(format!(
            "Ensure that `{}` provides all required properties and methods defined in the interface `{}`.",
            class_name.red().bold(),
            interface_name.red().bold()
        )),
        span:        None,
    })
}

fn suggest_property_not_assignable(err: &TsError) -> Option<Suggestion> {
    let property = extract_first_quoted(&err.message)?;
    let impl_type = extract_second_quoted(&err.message)?;
    let base_type = extract_third_quoted(&err.message)?;
    let property_impl_type = extract_quoted_value(&err.message, 7)?;
    let property_base_type = extract_quoted_value(&err.message, 9)?;

    Some(Suggestion {
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
            ),
        ],
        help:        Some(format!(
            "Ensure that the type of property `{}` in class `{}` is compatible with the type defined in base class `{}`.",
            property.red().bold(),
            impl_type.red().bold(),
            base_type.red().bold()
        )),
        span:        None,
    })
}

fn suggest_cannot_find_identifier(err: &TsError) -> Option<Suggestion> {
    let identifier = extract_first_quoted(&err.message)?;

    Some(Suggestion {
        suggestions: vec![format!(
            "Identifier `{}` can not be found in the current scope.",
            identifier.red().bold()
        )],
        help:        Some(format!(
            "Ensure that `{}` is declared and accessible in the current scope or remove this reference.",
            identifier.red().bold()
        )),
        span:        None,
    })
}

fn suggest_missing_return(_err: &TsError) -> Option<Suggestion> {
    Some(Suggestion {
        suggestions: vec!["A return value is missing where one is expected.".to_string()],
        help: Some(
            "A function that declares a return type must return a value of that type on all branches."
                .to_string(),
        ),
        span: None,
    })
}

fn suggest_uncallable_expression(err: &TsError) -> Option<Suggestion> {
    let expr = extract_first_quoted(&err.message)?;

    Some(Suggestion {
        suggestions: vec![format!(
            "Expression `{}` not can not be invoked or called.",
            expr.red().bold()
        )],
        help:        Some(format!(
            "Ensure that `{}` is a function or has a callable signature before invoking it.",
            expr.red().bold()
        )),
        span:        None,
    })
}

fn suggest_invalid_index_type(err: &TsError) -> Option<Suggestion> {
    let index_type = extract_first_quoted(&err.message)?;

    Some(Suggestion {
        suggestions: vec![format!(
            "`{}` can not be used as an index accessor.",
            index_type.red().bold()
        )],
        help: Some("Ensure that the index type is `number`, `string`, `symbol` or a compatible index type.".to_string()),
        span: None,
    })
}

fn suggest_invalid_index_signature(err: &TsError, tokens: &[Token]) -> Option<Suggestion> {
    let adjusted_column = err.column.saturating_sub(1);
    let token = find_token_at_position(tokens, err.line, adjusted_column);
    let span_text = token.map(|t| t.raw.clone())?;
    let span = token.map(|t| t.start..t.end)?;

    Some(Suggestion {
        suggestions: vec![format!(
            "`{}` is not a valid index type.",
            span_text.red().bold()
        )],
        help: Some("Ensure that the index type is `number`, `string`, `symbol`, `template literal` or a compatible index type.".to_string()),
        span: Some(span),
    })
}

fn suggest_typo_property(err: &TsError) -> Option<Suggestion> {
    let property_name = extract_first_quoted(&err.message)?;
    let type_name = extract_second_quoted(&err.message)?;
    let suggested_property_name = extract_third_quoted(&err.message)?;

    Some(Suggestion {
        suggestions: vec![format!(
            "Property `{}` does not exist on type `{}`. Try `{}` instead",
            property_name.red().bold(),
            type_name.yellow().bold(),
            suggested_property_name.green().bold()
        )],
        help:        Some(format!(
            "Check for typos in the property name `{}` or ensure that it is defined on type `{}`.",
            property_name.red().bold(),
            type_name.red().bold()
        )),
        span:        None,
    })
}

fn suggest_possibly_null(err: &TsError) -> Option<Suggestion> {
    let possible_null_var = extract_first_quoted(&err.message)?;

    Some(Suggestion {
        suggestions: vec![format!(
            "{} may be `null` here.",
            possible_null_var.red().bold()
        )],
        help:        Some(format!(
            "Consider optional chaining or an explicit null check before attempting to access `{}`",
            possible_null_var.red().bold()
        )),
        span:        None,
    })
}

fn suggest_object_unknown(err: &TsError) -> Option<Suggestion> {
    let unknown_var = extract_first_quoted(&err.message)?;

    Some(Suggestion {
        suggestions: vec![format!(
            "{} is of type `unknown`.",
            unknown_var.red().bold()
        )],
        help:        Some(format!(
            "Use type guards, type assertions, or narrow the type of `{}` before accessing its properties.",
            unknown_var.red().bold()
        )),
        span:        None,
    })
}

fn suggest_unterminated_string(err: &TsError) -> Option<Suggestion> {
    let literal = extract_first_quoted(&err.message)?;
    Some(Suggestion {
        suggestions: vec![format!(
            "String {} is missing \" to close the string.",
            literal.red().bold()
        )],
        help:        Some(
            "Ensure that all string literals are properly closed with matching quotes.".to_string(),
        ),
        span:        None,
    })
}

fn suggest_identifier_expected() -> Option<Suggestion> {
    Some(Suggestion {
        suggestions: vec!["An identifier was expected at this location in the code.".to_string()],
        help: Some(
            "Check the syntax near this location to ensure that an identifier is provided where required."
                .to_string(),
        ),
        span: None,
    })
}

fn suggest_disallowed_comma() -> Option<Suggestion> {
    Some(Suggestion {
        suggestions: vec!["Trailing commas are not allowed in this context.".to_string()],
        help:        Some("Remove the trailing comma to resolve the syntax error.".to_string()),
        span:        None,
    })
}

fn suggest_spread_parameter_last() -> Option<Suggestion> {
    Some(Suggestion {
        suggestions: vec![
            "A spread parameter must be the last parameter in a function signature.".to_string(),
        ],
        help:        Some(
            "Move the `...` parameter to the end of the list of parameters.".to_string(),
        ),
        span:        None,
    })
}

fn suggest_expression_expected() -> Option<Suggestion> {
    Some(Suggestion {
        suggestions: vec!["An expression was found but no value is assigned to it.".to_string()],
        help:        Some("Assign a value to the expression.".to_string()),
        span:        None,
    })
}

fn suggest_unique_members() -> Option<Suggestion> {
    Some(Suggestion {
        suggestions: vec!["Consider removing or renaming one of the object members".to_string()],
        help:        Some("An object may contain a member name once.".to_string()),
        span:        None,
    })
}

fn suggest_uninitialized_const(err: &TsError, tokens: &[Token]) -> Option<Suggestion> {
    let (name, span) = find_identifier_after_keyword(tokens, err.line, "const")?;

    Some(Suggestion {
        suggestions: vec![format!("`{}` must be initialized", name.red().bold())],
        help:        Some(format!(
            "Initialize `{}` with a value",
            name.yellow().bold()
        )),
        span:        Some(span),
    })
}

fn suggest_yield_not_in_generator() -> Option<Suggestion> {
    Some(Suggestion {
        suggestions: vec![format!(
            "`{}` can only be used in generator functions",
            "yield".red().bold()
        )],
        help:        Some(format!(
            "use `{}` inside of `{}`",
            "yield".yellow().bold(),
            "function*".yellow().bold()
        )),
        span:        None,
    })
}

fn suggest_jsx_flag() -> Option<Suggestion> {
    Some(Suggestion {
        suggestions: vec!["JSX can not be used.".to_string()],
        help:        Some(
            "Enable the JSX flag in your TypeScript configuration to use JSX syntax.".to_string(),
        ),
        span:        None,
    })
}

fn suggest_declared_unused(err: &TsError) -> Option<Suggestion> {
    let unused_decl = extract_first_quoted(&err.message)?;

    Some(Suggestion {
        suggestions: vec![format!("`{}` is unused", unused_decl.red().bold())],
        help:        Some(format!(
            "Consider removing the reference to `{}`",
            unused_decl.yellow().bold()
        )),
        span:        None,
    })
}

fn suggest_no_exported_member(err: &TsError) -> Option<Suggestion> {
    let non_exported_member = extract_quoted_value(&err.message, 3);
    let potential_correction = extract_quoted_value(&err.message, 5);

    Some(Suggestion {
        suggestions: vec![format!(
            "`{}` is not exported from the module.",
            non_exported_member?.red().bold()
        )],
        help:        Some(format!(
            "Did you mean to import `{}`?",
            potential_correction?.green().bold()
        )),
        span:        None,
    })
}

fn suggest_imported_unused() -> Option<Suggestion> {
    Some(Suggestion {
        suggestions: vec!["This import is unused".to_string()],
        help:        Some("Consider removing it".to_string()),
        span:        None,
    })
}

fn suggest_invalid_default_import() -> Option<Suggestion> {
    Some(Suggestion {
        suggestions: vec![format!(
            "`{}` is missing from compiler configuration, default imports are not allowed.",
            "esModuleInterop".red().bold()
        )],
        help:        Some(format!(
            "Enable compiler flag `{}` to allow default imports for this module.",
            "esModuleInterop".yellow().bold()
        )),
        span:        None,
    })
}
