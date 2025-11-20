use crate::parser::TsError;
use crate::tokenizer::Token;

/// Find the token at a specific position (line and column)
pub fn find_token_at_position<'a>(
    tokens: &'a [Token],
    line: usize,
    column: usize,
) -> Option<&'a Token> {
    tokens.iter().find(|token| {
        token.line == line
            && column >= token.column
            && column < token.column + token.raw.chars().count()
    })
}

/// Find a function/identifier token before the given position (searches backwards)
pub fn find_function_name_before<'a>(
    tokens: &'a [Token],
    line: usize,
    column: usize,
) -> Option<&'a Token> {
    // Search backwards from the error position for an identifier before a '('
    let mut found_paren = false;

    for token in tokens.iter().rev() {
        // Skip tokens after our position
        if token.line > line || (token.line == line && token.column >= column) {
            continue;
        }

        // Look for opening parenthesis first
        if token.raw == "(" {
            found_paren = true;
            continue;
        }

        // After finding '(', look for the identifier (function name)
        if found_paren && token.kind == crate::tokenizer::TokenKind::Identifier {
            return Some(token);
        }
    }

    None
}

/// Extract the identifier/token text at the error position
pub fn extract_identifier_at_error(err: &TsError, tokens: &[Token]) -> Option<String> {
    // Adjust column by -1 to match token indexing
    let adjusted_column = err.column.saturating_sub(1);
    find_token_at_position(tokens, err.line, adjusted_column).map(|token| token.raw.clone())
}

/// Extract the identifier at error position with a fallback default value
pub fn extract_identifier_or_default(err: &TsError, tokens: &[Token], default: &str) -> String {
    extract_identifier_at_error(err, tokens).unwrap_or_else(|| default.to_string())
}

/// Extract function name for parameter-related errors by searching backwards
pub fn extract_function_name(err: &TsError, tokens: &[Token], default: &str) -> String {
    find_function_name_before(tokens, err.line, err.column.saturating_sub(1))
        .map(|token| token.raw.clone())
        .unwrap_or_else(|| default.to_string())
}

/// Find the identifier token after a keyword on the given line
/// Returns both the identifier name and its span
pub fn find_identifier_after_keyword(
    tokens: &[Token],
    line: usize,
    keyword: &str,
) -> Option<(String, std::ops::Range<usize>)> {
    let mut found_keyword = false;

    for token in tokens.iter() {
        if token.line != line {
            if found_keyword {
                break;
            }
            continue;
        }

        if token.raw == keyword {
            found_keyword = true;
            continue;
        }

        if found_keyword && !token.raw.is_empty() && token.raw != ";" && token.raw != ":" {
            return Some((token.raw.clone(), token.start..token.end));
        }
    }

    None
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tokenizer::TokenKind;

    #[test]
    fn test_find_token_at_position() {
        let tokens = vec![
            Token {
                kind: TokenKind::Keyword,
                raw: "let".to_string(),
                start: 0,
                end: 3,
                line: 1,
                column: 0,
            },
            Token {
                kind: TokenKind::Identifier,
                raw: "x".to_string(),
                start: 4,
                end: 5,
                line: 1,
                column: 4,
            },
        ];

        assert_eq!(
            find_token_at_position(&tokens, 1, 0).map(|t| &t.raw),
            Some(&"let".to_string())
        );
        assert_eq!(
            find_token_at_position(&tokens, 1, 4).map(|t| &t.raw),
            Some(&"x".to_string())
        );
        assert_eq!(find_token_at_position(&tokens, 1, 10), None);
    }
}
