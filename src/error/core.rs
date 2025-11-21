/// Represents a TypeScript error from the compiler
#[derive(Debug, Clone)]
pub struct TsError {
    pub file: String,
    pub line: usize,
    pub column: usize,
    pub code: ErrorCode,
    pub message: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ErrorCode {
    // type errors
    TypeMismatch,
    InlineTypeMismatch,
    PropertyMissingInType,
    PropertyDoesNotExist,

    // null safety
    ObjectIsPossiblyNull,
    ObjectIsPossiblyUndefined,
    ObjectIsUnknown,

    // Function/parameter errors (TS25xx range)
    MissingParameters,
    IncompatibleOverload,
    UncallableExpression,

    // Syntax errors (TS1xxx range)
    UnterminatedStringLiteral,
    IdentifierExpected,
    ExpressionExpected,
    DisallowedTrailingComma,
    SpreadParameterMustBeLast,

    // Module/import errors (TS23xx, TS6xxx)
    NonExistentModuleImport,
    NoExportedMember,
    InvalidDefaultImport,

    // Class/interface errors
    IncorrectInterfaceImplementation,
    PropertyInClassNotAssignableToBase,
    ReadonlyPropertyAssignment,

    // Misc
    NoImplicitAny,
    UnintentionalComparison,
    DirectCastPotentiallyMistaken,
    SpreadArgumentMustBeTupleType,
    RightSideArithmeticMustBeEnumberable,
    LeftSideArithmeticMustBeEnumberable,
    InvalidShadowInScope,
    CannotFindIdentifier,
    MissingReturnValue,
    InvalidIndexType,
    InvalidIndexTypeSignature,
    TypoPropertyOnType,
    UniqueObjectMemberNames,
    UninitializedConst,
    YieldNotInGenerator,
    JsxFlagNotProvided,
    DeclaredButNeverUsed,
    ImportedButNeverUsed,

    /// Catch-all for unsupported error codes
    Unsupported(u16),
}
