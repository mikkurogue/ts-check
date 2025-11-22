#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ErrorCode {
    // type errors
    TypeMismatch,
    InlineTypeMismatch,
    PropertyMissingInType,
    PropertyDoesNotExist,
    TypeAssertionInJsNotAllowed,
    MappedTypeMustBeStatic,

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
    UnexpectedKeywordOrIdentifier,

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
    ElementImplicitAnyInvalidIndexTypeForObject,
    TypoPropertyOnType,
    UniqueObjectMemberNames,
    UninitializedConst,
    YieldNotInGenerator,
    DeclaredButNeverUsed,
    ImportedButNeverUsed,
    UnreachableCode,
    ConstEnumsDisallowed,

    // JSX related
    JsxFlagNotProvided,
    MissingJsxIntrinsicElementsDeclaration,
    JsxModuleNotSet,

    /// Catch-all for unsupported error codes
    Unsupported(u16),
}

impl ErrorCode {
    /// Create an `ErrorCode` from a string represenatation like "TS2322"
    pub fn from_str(code: &str) -> Self {
        match code {
            "TS2322" => ErrorCode::TypeMismatch,
            "TS2345" => ErrorCode::InlineTypeMismatch,
            "TS2554" => ErrorCode::MissingParameters,
            "TS7006" | "TS7044" => ErrorCode::NoImplicitAny,
            "TS2741" => ErrorCode::PropertyMissingInType,
            "TS2367" => ErrorCode::UnintentionalComparison,
            "TS18046" => ErrorCode::ObjectIsUnknown,
            "TS2339" => ErrorCode::PropertyDoesNotExist,
            "TS2532" | "TS18048" => ErrorCode::ObjectIsPossiblyUndefined,
            "TS2531" | "TS18047" => ErrorCode::ObjectIsPossiblyNull,
            "TS2352" => ErrorCode::DirectCastPotentiallyMistaken,
            "TS2556" => ErrorCode::SpreadArgumentMustBeTupleType,
            "TS2362" => ErrorCode::LeftSideArithmeticMustBeEnumberable,
            "TS2363" => ErrorCode::RightSideArithmeticMustBeEnumberable,
            "TS2394" => ErrorCode::IncompatibleOverload,
            "TS2451" => ErrorCode::InvalidShadowInScope,
            "TS2307" => ErrorCode::NonExistentModuleImport,
            "TS2540" => ErrorCode::ReadonlyPropertyAssignment,
            "TS2420" => ErrorCode::IncorrectInterfaceImplementation,
            "TS2416" => ErrorCode::PropertyInClassNotAssignableToBase,
            "TS2304" => ErrorCode::CannotFindIdentifier,
            "TS2355" => ErrorCode::MissingReturnValue,
            "TS2349" => ErrorCode::UncallableExpression,
            "TS2551" => ErrorCode::TypoPropertyOnType,
            "TS2538" => ErrorCode::InvalidIndexType,
            "TS1268" => ErrorCode::InvalidIndexTypeSignature,
            "TS1002" => ErrorCode::UnterminatedStringLiteral,
            "TS1003" => ErrorCode::IdentifierExpected,
            "TS1009" => ErrorCode::DisallowedTrailingComma,
            "TS1014" => ErrorCode::SpreadParameterMustBeLast,
            "TS1109" => ErrorCode::ExpressionExpected,
            "TS1117" => ErrorCode::UniqueObjectMemberNames,
            "TS1155" => ErrorCode::UninitializedConst,
            "TS1163" => ErrorCode::YieldNotInGenerator,
            "TS17004" => ErrorCode::JsxFlagNotProvided,
            "TS6133" => ErrorCode::DeclaredButNeverUsed,
            "TS2305" | "TS2724" => ErrorCode::NoExportedMember,
            "TS6192" => ErrorCode::ImportedButNeverUsed,
            "TS1259" => ErrorCode::InvalidDefaultImport,
            "TS95050" => ErrorCode::UnreachableCode,
            "TS8016" | "TS8010" => ErrorCode::TypeAssertionInJsNotAllowed,
            "TS7061" => ErrorCode::MappedTypeMustBeStatic,
            "TS7053" => ErrorCode::ElementImplicitAnyInvalidIndexTypeForObject,
            "TS7026" => ErrorCode::MissingJsxIntrinsicElementsDeclaration,
            "TS6244" => ErrorCode::ConstEnumsDisallowed,
            "TS6142" => ErrorCode::JsxModuleNotSet,
            "TS1434" => ErrorCode::UnexpectedKeywordOrIdentifier,

            other => {
                if let Some(num_str) = other.strip_prefix("TS")
                    && let Ok(num) = num_str.parse::<u16>()
                {
                    return ErrorCode::Unsupported(num);
                }
                ErrorCode::Unsupported(0)
            }
        }
    }

    /// Create the strng representation like "TS2322" from an `ErrorCode`
    pub fn as_str(&self) -> &'static str {
        match self {
            ErrorCode::TypeMismatch => "TS2322",
            ErrorCode::InlineTypeMismatch => "TS2345",
            ErrorCode::MissingParameters => "TS2554",
            ErrorCode::NoImplicitAny => "TS7006",
            ErrorCode::PropertyMissingInType => "TS2741",
            ErrorCode::UnintentionalComparison => "TS2367",
            ErrorCode::PropertyDoesNotExist => "TS2339",
            ErrorCode::ObjectIsPossiblyUndefined => "TS2532",
            ErrorCode::ObjectIsPossiblyNull => "TS2531",
            ErrorCode::ObjectIsUnknown => "TS18046",
            ErrorCode::DirectCastPotentiallyMistaken => "TS2352",
            ErrorCode::SpreadArgumentMustBeTupleType => "TS2556",
            ErrorCode::RightSideArithmeticMustBeEnumberable => "TS2363",
            ErrorCode::LeftSideArithmeticMustBeEnumberable => "TS2362",
            ErrorCode::IncompatibleOverload => "TS2394",
            ErrorCode::InvalidShadowInScope => "TS2451",
            ErrorCode::NonExistentModuleImport => "TS2307",
            ErrorCode::ReadonlyPropertyAssignment => "TS2540",
            ErrorCode::IncorrectInterfaceImplementation => "TS2420",
            ErrorCode::PropertyInClassNotAssignableToBase => "TS2416",
            ErrorCode::CannotFindIdentifier => "TS2304",
            ErrorCode::MissingReturnValue => "TS2355",
            ErrorCode::UncallableExpression => "TS2349",
            ErrorCode::InvalidIndexType => "TS2538",
            ErrorCode::InvalidIndexTypeSignature => "TS1268",
            ErrorCode::TypoPropertyOnType => "TS2551",
            ErrorCode::UnterminatedStringLiteral => "TS1002",
            ErrorCode::IdentifierExpected => "TS1003",
            ErrorCode::DisallowedTrailingComma => "TS1009",
            ErrorCode::SpreadParameterMustBeLast => "TS1014",
            ErrorCode::ExpressionExpected => "TS1109",
            ErrorCode::UniqueObjectMemberNames => "TS1117",
            ErrorCode::UninitializedConst => "TS1155",
            ErrorCode::YieldNotInGenerator => "TS1163",
            ErrorCode::JsxFlagNotProvided => "TS17004",
            ErrorCode::DeclaredButNeverUsed => "TS6133",
            ErrorCode::ImportedButNeverUsed => "TS6192",
            ErrorCode::NoExportedMember => "TS2305",
            ErrorCode::InvalidDefaultImport => "TS1259",
            ErrorCode::UnreachableCode => "TS95050",
            ErrorCode::TypeAssertionInJsNotAllowed => "TS8016",
            ErrorCode::MappedTypeMustBeStatic => "TS7061",
            ErrorCode::ElementImplicitAnyInvalidIndexTypeForObject => "TS7053",
            ErrorCode::MissingJsxIntrinsicElementsDeclaration => "TS7026",
            ErrorCode::ConstEnumsDisallowed => "TS6244",
            ErrorCode::JsxModuleNotSet => "TS6142",
            ErrorCode::UnexpectedKeywordOrIdentifier => "TS1434",
            ErrorCode::Unsupported(_) => {
                // This will return a static string for known codes, but for unsupported codes,
                // we return a dynamically allocated string. To keep the return type consistent,
                // we can return a placeholder here.
                "TS<unsupported>"
            }
        }
    }
}

impl std::fmt::Display for ErrorCode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}
