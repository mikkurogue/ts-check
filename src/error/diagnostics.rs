use crate::{error::core::TsError, suggestion::Suggestion, tokenizer::Token};

/// Trait that implements diagnostics for TS Errors
pub trait ErrorDiagnostic {
    /// Generate the suggestion for the error
    fn suggest(&self, err: &TsError, tokens: &[Token]) -> Option<Suggestion>;
}
