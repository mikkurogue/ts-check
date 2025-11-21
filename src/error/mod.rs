pub mod codes;
pub mod core;
pub mod diagnostics;

pub use core::{ErrorCode, TsError};
pub use diagnostics::ErrorDiagnostic;

/// Parse a TSC error line
pub fn parse(line: &str) -> Option<TsError> {
    let (file, rest) = line.split_once('(')?;
    let (coords, rest) = rest.split_once("): error ")?;
    let (line_s, col_s) = coords.split_once(',')?;
    let (code, msg) = rest.split_once(": ")?;

    Some(TsError {
        file: file.to_string(),
        line: usize::from_str_radix(line_s, 10).ok()?,
        column: usize::from_str_radix(col_s, 10).ok()?,
        code: ErrorCode::from_str(code),
        message: msg.to_string(),
    })
}
