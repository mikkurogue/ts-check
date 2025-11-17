#[derive(Debug, Clone, PartialEq)]
pub enum TokenKind {
    Identifier,
    Keyword,
    Symbol,
    TypeAnnotator,
    RightAngle,
    LeftAngle,
    LeftParen,
    RightParen,
    Comma,
    Literal,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Token {
    pub kind: TokenKind,
    pub raw: String,
    pub start: usize,
    pub end: usize,
    pub line: usize,
    pub column: usize,
}

#[derive(Debug, Clone)]
pub struct Tokenizer {
    /// The source code being tokenized from file reader
    pub src: String,
    pub position: usize,
    pub line: usize,
    pub column: usize,
}

impl Tokenizer {
    pub fn new(src: String) -> Self {
        Self {
            src,
            position: 0,
            line: 1,
            column: 0,
        }
    }

    fn skip_stuff(&mut self) {
        loop {
            let start_pos = self.position;

            // Skip whitespace
            while let Some(c) = self.src.get(self.position..).and_then(|s| s.chars().next()) {
                if c.is_whitespace() {
                    self.position += c.len_utf8();
                    if c == '\n' {
                        self.line += 1;
                        self.column = 0;
                    } else {
                        self.column += 1;
                    }
                } else {
                    break;
                }
            }

            // Skip comments
            if self
                .src
                .get(self.position..)
                .map_or(false, |s| s.starts_with("//"))
            {
                while let Some(c) = self.src.get(self.position..).and_then(|s| s.chars().next()) {
                    self.position += c.len_utf8();
                    if c == '\n' {
                        self.line += 1;
                        self.column = 0;
                        break;
                    }
                }
            }

            if self.position == start_pos {
                break;
            }
        }
    }

    fn read_identifier(&mut self) -> String {
        let start = self.position;
        while let Some(c) = self.src.get(self.position..).and_then(|s| s.chars().next()) {
            if c.is_alphanumeric() || c == '_' {
                self.position += c.len_utf8();
                self.column += 1;
            } else {
                break;
            }
        }
        self.src[start..self.position].to_string()
    }

    fn read_number(&mut self) -> String {
        let start = self.position;
        while let Some(c) = self.src.get(self.position..).and_then(|s| s.chars().next()) {
            if c.is_digit(10) || c == '.' {
                self.position += c.len_utf8();
                self.column += 1;
            } else {
                break;
            }
        }
        self.src[start..self.position].to_string()
    }

    fn read_string_literal(&mut self, quote: char) {
        while let Some(c) = self.src.get(self.position..).and_then(|s| s.chars().next()) {
            self.position += c.len_utf8();
            self.column += 1;
            if c == '\\' {
                // skip next char
                if let Some(next_c) = self.src.get(self.position..).and_then(|s| s.chars().next()) {
                    self.position += next_c.len_utf8();
                    self.column += 1;
                }
                continue;
            }
            if c == quote {
                return;
            }
        }
    }

    fn next(&mut self) -> Option<Token> {
        self.skip_stuff();

        if self.position >= self.src.len() {
            return None;
        }

        let start = self.position;
        let start_line = self.line;
        let start_col = self.column;

        let c = self
            .src
            .get(self.position..)
            .and_then(|s| s.chars().next())?;

        let kind;

        match c {
            '(' => {
                self.position += 1;
                self.column += 1;
                kind = TokenKind::LeftParen;
            }
            ')' => {
                self.position += 1;
                self.column += 1;
                kind = TokenKind::RightParen;
            }
            '{' | '}' | '[' | ']' | ';' | '.' => {
                self.position += 1;
                self.column += 1;
                kind = TokenKind::Symbol;
            }
            ':' => {
                self.position += 1;
                self.column += 1;
                kind = TokenKind::TypeAnnotator;
            }
            ',' => {
                self.position += 1;
                self.column += 1;
                kind = TokenKind::Comma;
            }
            '<' => {
                self.position += 1;
                self.column += 1;
                kind = TokenKind::LeftAngle;
            }
            '>' => {
                self.position += 1;
                self.column += 1;
                kind = TokenKind::RightAngle;
            }
            '"' | '\'' | '`' => {
                self.position += 1; // opening quote
                self.column += 1;
                self.read_string_literal(c);
                kind = TokenKind::Literal;
            }
            c if c.is_alphabetic() || c == '_' => {
                let ident = self.read_identifier();
                if is_keyword(&ident) {
                    kind = TokenKind::Keyword;
                } else {
                    kind = TokenKind::Identifier;
                }
            }
            c if c.is_digit(10) => {
                self.read_number();
                kind = TokenKind::Literal;
            }
            _ => {
                self.position += c.len_utf8();
                self.column += 1;
                kind = TokenKind::Symbol;
            }
        };

        let end = self.position;
        let raw = self.src[start..end].to_string();

        Some(Token {
            kind,
            raw,
            start,
            end,
            line: start_line,
            column: start_col,
        })
    }

    pub fn tokenize(&mut self) -> Vec<Token> {
        let mut tokens = Vec::new();
        while let Some(token) = self.next() {
            tokens.push(token);
        }
        tokens
    }
}

fn is_keyword(s: &str) -> bool {
    matches!(
        s,
        "function"
            | "let"
            | "const"
            | "var"
            | "if"
            | "else"
            | "return"
            | "true"
            | "false"
            | "type"
            | "interface"
            | "class"
            | "enum"
            | "export"
            | "import"
            | "from"
            | "as"
            | "new"
            | "while"
            | "for"
            | "in"
            | "of"
            | "do"
            | "switch"
            | "case"
            | "default"
            | "break"
            | "continue"
            | "try"
            | "catch"
            | "finally"
            | "throw"
            | "debugger"
            | "with"
            | "async"
            | "await"
            | "public"
            | "private"
            | "protected"
            | "static"
            | "readonly"
    )
}
