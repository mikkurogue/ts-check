#[derive(Debug, Clone)]
pub struct Suggestion {
    pub suggestions: Vec<String>,
    pub help: Option<String>,
    pub span: Option<std::ops::Range<usize>>,
}
