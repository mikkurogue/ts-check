use crate::message_parser::{
    extract_first_quoted, extract_quoted_value, extract_second_quoted, extract_third_quoted,
    parse_property_missing_error, parse_ts2322_error, parse_ts2345_error,
};
use crate::parser::{CommonErrors, TsError};
use crate::token_utils::{
    extract_function_name, extract_identifier_at_error, extract_identifier_or_default,
    find_identifier_after_keyword, find_token_at_position,
};
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
    pub span: Option<std::ops::Range<usize>>,
}

trait SuggestionHandler {
    fn handle(&self, err: &TsError, tokens: &[Token]) -> Option<Suggestion>;
}

struct TypeMismatchHandler;
impl SuggestionHandler for TypeMismatchHandler {
    fn handle(&self, err: &TsError, _tokens: &[Token]) -> Option<Suggestion> {
        Some(Suggestion {
            suggestions: vec![type_mismatch_2322(err)?],
            help: Some(
                "Ensure that the types are compatible or perform an explicit conversion."
                    .to_string(),
            ),
            span: None,
        })
    }
}

struct InlineTypeMismatchHandler;
impl SuggestionHandler for InlineTypeMismatchHandler {
    fn handle(&self, err: &TsError, _tokens: &[Token]) -> Option<Suggestion> {
        // Check if this is a callback signature mismatch (too many/few parameters)
        if err
            .message
            .contains("Target signature provides too few arguments")
        {
            // Parse: "Expected X or more, but got Y"
            let (expected, got) = if let Some(expected_str) = err.message.split("Expected ").nth(1)
            {
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
                "The callback function has too many parameters for the expected signature."
                    .to_string()
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
            let suggestion =
                "The callback function has too few parameters for the expected signature."
                    .to_string();

            return Some(Suggestion {
                suggestions: vec![suggestion],
                help: Some(
                    "Add the missing parameters to the callback function to match the expected signature.".to_string()
                ),
                span: None,
            });
        }

        // Otherwise, try to parse object property mismatches
        let suggestions = inline_type_mismatch_2345(err);
        Some(Suggestion {
            suggestions: suggestions.unwrap_or_else(|| {
                vec!["Argument type does not match the expected parameter type.".to_string()]
            }),
            help: Some(
                "Check the function arguments to ensure they match the expected parameter types."
                    .to_string(),
            ),
            span: None,
        })
    }
}

struct MissingParametersHandler;
impl SuggestionHandler for MissingParametersHandler {
    fn handle(&self, err: &TsError, tokens: &[Token]) -> Option<Suggestion> {
        // For TS2554, the error is typically on the function being called with wrong args
        // First try to get the identifier at the error position
        let fn_name = if let Some(name) = extract_identifier_at_error(err, tokens) {
            if !name.is_empty() {
                name
            } else {
                // Fallback to extracting from message, then searching backwards
                let fallback =
                    extract_first_quoted(&err.message).unwrap_or_else(|| "function".to_string());
                extract_function_name(err, tokens, &fallback)
            }
        } else {
            // Fallback to extracting from message, then searching backwards
            let fallback =
                extract_first_quoted(&err.message).unwrap_or_else(|| "function".to_string());
            extract_function_name(err, tokens, &fallback)
        };

        // Parse expected/actual counts from message
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
            help: Some(help),
            span: None,
        })
    }
}

struct NoImplicitAnyHandler;
impl SuggestionHandler for NoImplicitAnyHandler {
    fn handle(&self, err: &TsError, _tokens: &[Token]) -> Option<Suggestion> {
        let param_name =
            extract_first_quoted(&err.message).unwrap_or_else(|| "parameter".to_string());

        Some(Suggestion {
            suggestions: vec![format!("{} is implicitly `any`.", param_name.red().bold())],
            help: Some(
                "Consider adding type annotations to avoid implicit 'any' types.".to_string(),
            ),
            span: None,
        })
    }
}

struct PropertyMissingInTypeHandler;
impl SuggestionHandler for PropertyMissingInTypeHandler {
    fn handle(&self, err: &TsError, tokens: &[Token]) -> Option<Suggestion> {
        if let Some(type_name) = parse_property_missing_error(&err.message) {
            let var_name = extract_identifier_or_default(err, tokens, "");

            Some(Suggestion {
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
                span: None,
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
}

struct UnintentionalComparisonHandler;
impl SuggestionHandler for UnintentionalComparisonHandler {
    fn handle(&self, _err: &TsError, _tokens: &[Token]) -> Option<Suggestion> {
        Some(Suggestion {
            suggestions: vec![
                "Impossible to compare as left side value is narrowed to a single value."
                    .to_string(),
            ],
            help: Some("Review the comparison logic to ensure it makes sense.".to_string()),
            span: None,
        })
    }
}

struct PropertyDoesNotExistHandler;
impl SuggestionHandler for PropertyDoesNotExistHandler {
    fn handle(&self, err: &TsError, _tokens: &[Token]) -> Option<Suggestion> {
        let property_name =
            extract_first_quoted(&err.message).unwrap_or_else(|| "property".to_string());
        let type_name = extract_second_quoted(&err.message).unwrap_or_else(|| "type".to_string());

        Some(Suggestion {
            suggestions: vec![format!(
                "Property `{}` is not found on type `{}`.",
                property_name.red().bold(),
                type_name.red().bold()
            )],
            help: Some(
                "Ensure the property exists on the type or adjust your code to avoid accessing it."
                    .to_string(),
            ),
            span: None,
        })
    }
}

struct ObjectIsPossiblyUndefinedHandler;
impl SuggestionHandler for ObjectIsPossiblyUndefinedHandler {
    fn handle(&self, err: &TsError, _tokens: &[Token]) -> Option<Suggestion> {
        let possible_undefined_var =
            extract_first_quoted(&err.message).unwrap_or_else(|| "object".to_string());

        Some(Suggestion {
            suggestions: vec![format!(
                "{} may be `undefined` here.",
                possible_undefined_var.red().bold()
            )],
            help: Some(format!(
                "Consider optional chaining or an explicit check before attempting to access `{}`",
                possible_undefined_var.red().bold()
            )),
            span: None,
        })
    }
}

struct DirectCastPotentiallyMistakenHandler;
impl SuggestionHandler for DirectCastPotentiallyMistakenHandler {
    fn handle(&self, err: &TsError, _tokens: &[Token]) -> Option<Suggestion> {
        let cast_from_type =
            extract_first_quoted(&err.message).unwrap_or_else(|| "type".to_string());
        let cast_to_type =
            extract_second_quoted(&err.message).unwrap_or_else(|| "type".to_string());

        Some(Suggestion {
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
            span: None,
        })
    }
}

struct SpreadArgumentMustBeTupleTypeHandler;
impl SuggestionHandler for SpreadArgumentMustBeTupleTypeHandler {
    fn handle(&self, _err: &TsError, _tokens: &[Token]) -> Option<Suggestion> {
        Some(Suggestion {
            suggestions: vec![
                "The argument being spread must be a tuple type or a `spreadable` type."
                    .to_string()
            ],
            help: Some(
                "Ensure that the argument being spread is a tuple type compatible with the function's parameter type."
                    .to_string(),
            ),
            span: None,
        })
    }
}

struct RightSideArithmeticMustBeEnumberableHandler;
impl SuggestionHandler for RightSideArithmeticMustBeEnumberableHandler {
    fn handle(&self, _err: &TsError, _tokens: &[Token]) -> Option<Suggestion> {
        Some(Suggestion {
            suggestions: vec![
                "The right-hand side of any arithmetic operation must be a number or enumerable."
                    .to_string()
            ],
            help: Some(
                "Ensure that the value on the right side of the arithmetic operator is of type `number`, `bigint` or an enum member."
                    .to_string(),
            ),
            span: None,
        })
    }
}

struct LeftSideArithmeticMustBeEnumberableHandler;
impl SuggestionHandler for LeftSideArithmeticMustBeEnumberableHandler {
    fn handle(&self, _err: &TsError, _tokens: &[Token]) -> Option<Suggestion> {
        Some(Suggestion {
            suggestions: vec![
                "The left-hand side of any arithmetic operation must be a number or enumerable."
                    .to_string()
            ],
            help: Some(
                "Ensure that the value on the left side of the arithmetic operator is of type `number`, `bigint` or an enum member."
                    .to_string(),
            ),
            span: None,
        })
    }
}

struct IncompatibleOverloadHandler;
impl SuggestionHandler for IncompatibleOverloadHandler {
    fn handle(&self, _err: &TsError, _tokens: &[Token]) -> Option<Suggestion> {
        Some(Suggestion {
            suggestions: vec![
                "The provided arguments do not match any overload of the function."
                    .to_string()
            ],
            help: Some(
                "Check the function overloads and ensure that this signature adheres to the parent signature."
                    .to_string(),
            ),
            span: None,
        })
    }
}

struct InvalidShadowInScopeHandler;
impl SuggestionHandler for InvalidShadowInScopeHandler {
    fn handle(&self, err: &TsError, _tokens: &[Token]) -> Option<Suggestion> {
        let var_name = extract_first_quoted(&err.message).unwrap_or_else(|| "variable".to_string());

        Some(Suggestion {
            suggestions: vec![format!(
                "Declared variable `{}` can not shadow another variable in this scope.",
                var_name.red().bold()
            )],
            help: Some(format!(
                "Consider renaming the invalid shadowed variable `{}`.",
                var_name.red().bold()
            )),
            span: None,
        })
    }
}

struct NonExistentModuleImportHandler;
impl SuggestionHandler for NonExistentModuleImportHandler {
    fn handle(&self, err: &TsError, _tokens: &[Token]) -> Option<Suggestion> {
        let module_name =
            extract_first_quoted(&err.message).unwrap_or_else(|| "module".to_string());

        Some(Suggestion {
            suggestions: vec![format!(
                "Module `{}` does not exist.",
                module_name.red().bold()
            )],
            help: Some(format!(
                "Ensure that the module `{}` is installed and the import path is correct.",
                module_name.red().bold(),
            )),
            span: None,
        })
    }
}

struct ReadonlyPropertyAssignmentHandler;
impl SuggestionHandler for ReadonlyPropertyAssignmentHandler {
    fn handle(&self, err: &TsError, _tokens: &[Token]) -> Option<Suggestion> {
        let property_name =
            extract_first_quoted(&err.message).unwrap_or_else(|| "property".to_string());

        Some(Suggestion {
            suggestions: vec![format!(
                "Property `{}` is readonly and thus can not be re-assigned.",
                property_name.red().bold()
            )],
            help: Some(format!(
                "Consider removing the assignment to the read-only property `{}` or changing its declaration to be mutable.",
                property_name.red().bold()
            )),
            span: None,
        })
    }
}

struct IncorrectInterfaceImplementationHandler;
impl SuggestionHandler for IncorrectInterfaceImplementationHandler {
    fn handle(&self, err: &TsError, _tokens: &[Token]) -> Option<Suggestion> {
        let class_name = extract_first_quoted(&err.message).unwrap_or_else(|| "class".to_string());
        let interface_name =
            extract_second_quoted(&err.message).unwrap_or_else(|| "interface".to_string());
        let missing_property =
            extract_third_quoted(&err.message).unwrap_or_else(|| "property".to_string());

        Some(Suggestion {
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
            span: None,
        })
    }
}

struct PropertyInClassNotAssignableToBaseHandler;
impl SuggestionHandler for PropertyInClassNotAssignableToBaseHandler {
    fn handle(&self, err: &TsError, _tokens: &[Token]) -> Option<Suggestion> {
        let property = extract_first_quoted(&err.message).unwrap_or_else(|| "property".to_string());
        let impl_type = extract_second_quoted(&err.message).unwrap_or_else(|| "type".to_string());
        let base_type =
            extract_third_quoted(&err.message).unwrap_or_else(|| "base type".to_string());
        let property_impl_type =
            extract_quoted_value(&err.message, 7).unwrap_or_else(|| "type".to_string());
        let property_base_type =
            extract_quoted_value(&err.message, 9).unwrap_or_else(|| "base type".to_string());

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
            help: Some(format!(
                "Ensure that the type of property `{}` in class `{}` is compatible with the type defined in base class `{}`.",
                property.red().bold(),
                impl_type.red().bold(),
                base_type.red().bold()
            )),
            span: None,
        })
    }
}

struct CannotFindIdentifierHandler;
impl SuggestionHandler for CannotFindIdentifierHandler {
    fn handle(&self, err: &TsError, _tokens: &[Token]) -> Option<Suggestion> {
        let identifier =
            extract_first_quoted(&err.message).unwrap_or_else(|| "identifier".to_string());

        Some(Suggestion {
            suggestions: vec![format!(
                "Identifier `{}` cannot be found in the current scope.",
                identifier.red().bold()
            )],
            help: Some(format!(
                "Ensure that `{}` is declared and accessible in the current scope or remove this reference.",
                identifier.red().bold()
            )),
            span: None,
        })
    }
}

struct MissingReturnValueHandler;
impl SuggestionHandler for MissingReturnValueHandler {
    fn handle(&self, _err: &TsError, _tokens: &[Token]) -> Option<Suggestion> {
        Some(Suggestion {
            suggestions: vec![
                "A return value is missing where one is expected.".to_string()
            ],
            help: Some(
                "A function that declares a return type must return a value of that type on all branches."
                    .to_string(),
            ),
            span: None,
        })
    }
}

struct UncallableExpressionHandler;
impl SuggestionHandler for UncallableExpressionHandler {
    fn handle(&self, err: &TsError, _tokens: &[Token]) -> Option<Suggestion> {
        let expr = extract_first_quoted(&err.message).unwrap_or_else(|| "expression".to_string());

        Some(Suggestion {
            suggestions: vec![format!(
                "Expression `{}` not can not be invoked or called.",
                expr.red().bold()
            )],
            help: Some(format!(
                "Ensure that `{}` is a function or has a callable signature before invoking it.",
                expr.red().bold()
            )),
            span: None,
        })
    }
}

struct InvalidIndexTypeHandler;
impl SuggestionHandler for InvalidIndexTypeHandler {
    fn handle(&self, err: &TsError, _tokens: &[Token]) -> Option<Suggestion> {
        let index_type = extract_first_quoted(&err.message).unwrap_or_else(|| "type".to_string());

        Some(Suggestion {
            suggestions: vec![format!(
                "`{}` cannot be used as an index accessor.",
                index_type.red().bold()
            )],
            help: Some("Ensure that the index type is `number`, `string`, `symbol` or a compatible index type.".to_string()),
            span: None,
        })
    }
}

/// I think this is mostly to handle custom types like type MyType = { something: string}
struct InvalidIndexTypeSignatureHandler;
impl SuggestionHandler for InvalidIndexTypeSignatureHandler {
    fn handle(&self, err: &TsError, tokens: &[Token]) -> Option<Suggestion> {
        let adjusted_column = err.column.saturating_sub(1);
        let token = find_token_at_position(tokens, err.line, adjusted_column);
        let span_text = token
            .map(|t| t.raw.clone())
            .unwrap_or_else(|| "property".to_string());
        let span = token.map(|t| t.start..t.end).unwrap_or_else(|| 0..0);

        Some(Suggestion {
            suggestions: vec![format!(
                "`{}` is not a valid index type.",
                span_text.red().bold()
            )],
            help: Some("Ensure that the index type is `number`, `string`, `symbol`, `template literal` or a compatible index type.".to_string()),
            span: Some(span),
        })
    }
}

struct TypoPropertyOnTypeHandler;
impl SuggestionHandler for TypoPropertyOnTypeHandler {
    fn handle(&self, err: &TsError, _tokens: &[Token]) -> Option<Suggestion> {
        let property_name =
            extract_first_quoted(&err.message).unwrap_or_else(|| "property".to_string());
        let type_name = extract_second_quoted(&err.message).unwrap_or_else(|| "type".to_string());
        let suggested_property_name =
            extract_third_quoted(&err.message).unwrap_or_else(|| "property".to_string());

        Some(Suggestion {
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
            span: None,
        })
    }
}

struct ObjectIsPossiblyNullHandler;
impl SuggestionHandler for ObjectIsPossiblyNullHandler {
    fn handle(&self, err: &TsError, _tokens: &[Token]) -> Option<Suggestion> {
        let possible_null_var =
            extract_first_quoted(&err.message).unwrap_or_else(|| "object".to_string());

        Some(Suggestion {
            suggestions: vec![format!(
                "{} may be `null` here.",
                possible_null_var.red().bold()
            )],
            help: Some(format!(
                "Consider optional chaining or an explicit null check before attempting to access `{}`",
                possible_null_var.red().bold()
            )),
            span: None,
        })
    }
}

struct ObjectIsUnknownHandler;
impl SuggestionHandler for ObjectIsUnknownHandler {
    fn handle(&self, err: &TsError, _tokens: &[Token]) -> Option<Suggestion> {
        let unknown_var = extract_first_quoted(&err.message).unwrap_or_else(|| "value".to_string());

        Some(Suggestion {
            suggestions: vec![format!(
                "{} is of type `unknown`.",
                unknown_var.red().bold()
            )],
            help: Some(format!(
                "Use type guards, type assertions, or narrow the type of `{}` before accessing its properties.",
                unknown_var.red().bold()
            )),
            span: None,
        })
    }
}

struct UnterminatedStringLiteralHandler;
impl SuggestionHandler for UnterminatedStringLiteralHandler {
    fn handle(&self, err: &TsError, _tokens: &[Token]) -> Option<Suggestion> {
        let literal =
            extract_first_quoted(&err.message).unwrap_or_else(|| "string literal".to_string());
        Some(Suggestion {
            suggestions: vec![format!(
                "String {} is missing \" to close the string.",
                literal.red().bold()
            )],
            help: Some(
                "Ensure that all string literals are properly closed with matching quotes."
                    .to_string(),
            ),
            span: None,
        })
    }
}

struct IdentifierExpectedHandler;
impl SuggestionHandler for IdentifierExpectedHandler {
    fn handle(&self, _err: &TsError, _tokens: &[Token]) -> Option<Suggestion> {
        Some(Suggestion {
            suggestions: vec![
                "An identifier was expected at this location in the code.".to_string(),
            ],
            help: Some(format!(
                "Check the syntax near this location to ensure that an identifier is provided where required."
            )),
            span: None,
        })
    }
}

struct DisallowedTrailingCommaHandler;
impl SuggestionHandler for DisallowedTrailingCommaHandler {
    fn handle(&self, _err: &TsError, _tokens: &[Token]) -> Option<Suggestion> {
        Some(Suggestion {
            suggestions: vec!["Trailing commas are not allowed in this context.".to_string()],
            help: Some("Remove the trailing comma to resolve the syntax error.".to_string()),
            span: None,
        })
    }
}

struct SpreadParameterMustBeLastHandler;
impl SuggestionHandler for SpreadParameterMustBeLastHandler {
    fn handle(&self, _err: &TsError, _tokens: &[Token]) -> Option<Suggestion> {
        Some(Suggestion {
            suggestions: vec![
                "A spread parameter must be the last parameter in a function signature."
                    .to_string(),
            ],
            help: Some(
                "Move the `...` parameter to the end of the list of parameters.".to_string(),
            ),
            span: None,
        })
    }
}

struct ExpressionExpectedHandler;
impl SuggestionHandler for ExpressionExpectedHandler {
    fn handle(&self, _err: &TsError, _tokens: &[Token]) -> Option<Suggestion> {
        Some(Suggestion {
            suggestions: vec![
                "An expression was found but no value is assigned to it.".to_string(),
            ],
            help: Some("Assign a value to the expression.".to_string()),
            span: None,
        })
    }
}

struct UniqueObjectMemberNamesHandler;
impl SuggestionHandler for UniqueObjectMemberNamesHandler {
    fn handle(&self, _err: &TsError, _tokens: &[Token]) -> Option<Suggestion> {
        Some(Suggestion {
            suggestions: vec![
                "Consider removing or renaming one of the object members".to_string(),
            ],
            help: Some("An object may contain a member name once.".to_string()),
            span: None,
        })
    }
}

struct UninitializedConstHandler;
impl SuggestionHandler for UninitializedConstHandler {
    fn handle(&self, err: &TsError, tokens: &[Token]) -> Option<Suggestion> {
        // Find the identifier after 'const' keyword
        let (name, span) = find_identifier_after_keyword(tokens, err.line, "const")
            .unwrap_or_else(|| ("const".to_string(), 0..0));

        Some(Suggestion {
            suggestions: vec![format!("`{}` must be initialized", name.red().bold())],
            help: Some(format!(
                "Initialize `{}` with a value",
                name.yellow().bold()
            )),
            span: Some(span),
        })
    }
}

struct YieldNotInGeneratorHandler;
impl SuggestionHandler for YieldNotInGeneratorHandler {
    fn handle(&self, _err: &TsError, _tokens: &[Token]) -> Option<Suggestion> {
        Some(Suggestion {
            suggestions: vec![format!(
                "`{}` can only be used in generator functions",
                "yield".red().bold()
            )],
            help: Some(format!(
                "use `{}` inside of `{}`",
                "yield".yellow().bold(),
                "function*".yellow().bold()
            )),
            span: None,
        })
    }
}

impl Suggest for Suggestion {
    /// Build a suggestion and help text for the given TsError
    fn build(err: &TsError, tokens: &[Token]) -> Option<Self> {
        let handler: Box<dyn SuggestionHandler> = match err.code {
            CommonErrors::TypeMismatch => Box::new(TypeMismatchHandler),
            CommonErrors::InlineTypeMismatch => Box::new(InlineTypeMismatchHandler),
            CommonErrors::MissingParameters => Box::new(MissingParametersHandler),
            CommonErrors::NoImplicitAny => Box::new(NoImplicitAnyHandler),
            CommonErrors::PropertyMissingInType => Box::new(PropertyMissingInTypeHandler),
            CommonErrors::UnintentionalComparison => Box::new(UnintentionalComparisonHandler),
            CommonErrors::PropertyDoesNotExist => Box::new(PropertyDoesNotExistHandler),
            CommonErrors::ObjectIsPossiblyUndefined => Box::new(ObjectIsPossiblyUndefinedHandler),
            CommonErrors::DirectCastPotentiallyMistaken => {
                Box::new(DirectCastPotentiallyMistakenHandler)
            }
            CommonErrors::SpreadArgumentMustBeTupleType => {
                Box::new(SpreadArgumentMustBeTupleTypeHandler)
            }
            CommonErrors::RightSideArithmeticMustBeEnumberable => {
                Box::new(RightSideArithmeticMustBeEnumberableHandler)
            }
            CommonErrors::LeftSideArithmeticMustBeEnumberable => {
                Box::new(LeftSideArithmeticMustBeEnumberableHandler)
            }
            CommonErrors::IncompatibleOverload => Box::new(IncompatibleOverloadHandler),
            CommonErrors::InvalidShadowInScope => Box::new(InvalidShadowInScopeHandler),
            CommonErrors::NonExistentModuleImport => Box::new(NonExistentModuleImportHandler),
            CommonErrors::ReadonlyPropertyAssignment => Box::new(ReadonlyPropertyAssignmentHandler),
            CommonErrors::IncorrectInterfaceImplementation => {
                Box::new(IncorrectInterfaceImplementationHandler)
            }
            CommonErrors::PropertyInClassNotAssignableToBase => {
                Box::new(PropertyInClassNotAssignableToBaseHandler)
            }
            CommonErrors::CannotFindIdentifier => Box::new(CannotFindIdentifierHandler),
            CommonErrors::MissingReturnValue => Box::new(MissingReturnValueHandler),
            CommonErrors::UncallableExpression => Box::new(UncallableExpressionHandler),
            CommonErrors::InvalidIndexType => Box::new(InvalidIndexTypeHandler),
            CommonErrors::InvalidIndexTypeSignature => Box::new(InvalidIndexTypeSignatureHandler),
            CommonErrors::TypoPropertyOnType => Box::new(TypoPropertyOnTypeHandler),
            CommonErrors::ObjectIsPossiblyNull => Box::new(ObjectIsPossiblyNullHandler),
            CommonErrors::ObjectIsUnknown => Box::new(ObjectIsUnknownHandler),
            CommonErrors::UnterminatedStringLiteral => Box::new(UnterminatedStringLiteralHandler),
            CommonErrors::IdentifierExpected => Box::new(IdentifierExpectedHandler),
            CommonErrors::DisallowedTrailingComma => Box::new(DisallowedTrailingCommaHandler),
            CommonErrors::SpreadParameterMustBeLast => Box::new(SpreadParameterMustBeLastHandler),
            CommonErrors::ExpressionExpected => Box::new(ExpressionExpectedHandler),
            CommonErrors::UniqueObjectMemberNames => Box::new(UniqueObjectMemberNamesHandler),
            CommonErrors::UninitializedConst => Box::new(UninitializedConstHandler),
            CommonErrors::YieldNotInGenerator => Box::new(YieldNotInGeneratorHandler),
            CommonErrors::Unsupported(_) => return None,
        };

        handler.handle(err, tokens)
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
