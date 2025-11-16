use regex::Regex;

#[derive(Debug, Clone)]
pub struct TsError {
    pub file: String,
    pub line: usize,
    pub column: usize,
    pub code: String,
    pub message: String,
}

/// for now only supprts the following:
/// src/index.ts:10:5 - error TS2322: Type 'string' is not assignable to type 'number'.
/// this will also be our initial test case
pub fn parse(line: &str) -> Option<TsError> {
    // Matches: index.ts(1,7): error TS2322: Type 'string' is not assignable to type 'number'.
    let re = Regex::new(
        r#"^(?P<file>[^\(]+)\((?P<line>\d+),(?P<col>\d+)\): error (?P<code>TS\d+): (?P<msg>.*)$"#,
    )
    .ok()?;

    let caps = re.captures(line)?;

    Some(TsError {
        file: caps.name("file")?.as_str().to_string(),
        line: caps.name("line")?.as_str().parse().ok()?,
        column: caps.name("col")?.as_str().parse().ok()?,
        code: caps.name("code")?.as_str().to_string(),
        message: caps.name("msg")?.as_str().to_string(),
    })
}
