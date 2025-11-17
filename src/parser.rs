#[derive(Debug, Clone)]
pub struct TsError {
    pub file: String,
    pub line: usize,
    pub column: usize,
    pub code: CommonErrors,
    pub message: String,
}

#[derive(Debug, Clone)]
pub enum CommonErrors {
    TypeMismatch,
    MissingParameters,
    NoImplicitAny,
    PropertyMissingInType,
    UnintentionalComparison,
    Unsupported(String),
}

impl std::fmt::Display for CommonErrors {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            CommonErrors::TypeMismatch => write!(f, "TS2322"),
            CommonErrors::MissingParameters => write!(f, "TS2554"),
            CommonErrors::NoImplicitAny => write!(f, "TS7006"),
            CommonErrors::PropertyMissingInType => write!(f, "TS2741"),
            CommonErrors::UnintentionalComparison => write!(f, "TS2367"),
            CommonErrors::Unsupported(code) => write!(f, "{}", code),
        }
    }
}

impl CommonErrors {
    pub fn from_code(code: &str) -> CommonErrors {
        match code {
            "TS2322" => CommonErrors::TypeMismatch,
            "TS2554" => CommonErrors::MissingParameters,
            "TS7006" | "TS7044" => CommonErrors::NoImplicitAny,
            "TS2741" => CommonErrors::PropertyMissingInType,
            "TS2367" => CommonErrors::UnintentionalComparison,
            other => CommonErrors::Unsupported(other.to_string()),
        }
    }
}

pub fn parse(line: &str) -> Option<TsError> {
    let (file, rest) = line.split_once('(')?;
    let (coords, rest) = rest.split_once("): error ")?;
    let (line_s, col_s) = coords.split_once(',')?;
    let (code, msg) = rest.split_once(": ")?;

    Some(TsError {
        file: file.to_string(),
        line: usize::from_str_radix(line_s, 10).ok()?,
        column: usize::from_str_radix(col_s, 10).ok()?,
        code: CommonErrors::from_code(code),
        message: msg.to_string(),
    })
}
