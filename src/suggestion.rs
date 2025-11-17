use crate::parser::{CommonErrors, TsError};

pub trait Suggest {
    fn build(err: &TsError) -> Option<String>;
}

pub struct Suggestion;

impl Suggest for Suggestion {
    fn build(err: &TsError) -> Option<String> {
        let suggestion = match err.code {
            CommonErrors::TypeMismatch => type_mismatch(err),
            CommonErrors::MissingParameters => Some(
                "Check if all required parameters are provided in the function call.".to_string(),
            ),
            CommonErrors::NoImplicitAny => Some(
                "Consider adding explicit type annotations to avoid implicit `any` types."
                    .to_string(),
            ),
            CommonErrors::PropertyMissingInType => Some(
                "Verify that the object structure includes all required members of the specified type."
                    .to_string(),
            ),
            CommonErrors::UnintentionalComparison => Some(
                "Impossible to compare as left side value is narrowed to a single value."
                    .to_string(),
            ),
            CommonErrors::Unsupported(_) => None,
        };
        suggestion
    }
}

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
