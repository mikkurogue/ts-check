/// Represents a TypeScript error from the compiler
#[derive(Debug, Clone)]
pub struct TsError {
    pub file: String,
    pub line: usize,
    pub column: usize,
    pub code: super::codes::ErrorCode,
    pub message: String,
}
