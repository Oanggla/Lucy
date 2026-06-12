#[derive(Debug, Clone, PartialEq)]
pub enum TokenKind {
    // Keywords
    Let,
    Fn,
    While,
    If,
    Else,
    True,
    False,

    // Literals
    Identifier(String),
    Number(f64),
    String(String),

    // Operators
    Plus,
    Minus,
    Asterisk,
    Slash,
    Percent,
    Assign,
    Eq,
    Ne,
    Lt,
    Gt,
    Le,
    Ge,
    Pipeline, // |>

    // Punctuation
    LParen,
    RParen,
    LBrace,
    RBrace,
    Comma,
    Semicolon,

    EOF,
}

#[derive(Debug, Clone)]
pub struct Token {
    pub kind: TokenKind,
    pub line: usize,
    pub column: usize,
}

pub struct Lexer<'a> {
    _input: &'a str,
    chars: Vec<char>,
    position: usize,
    line: usize,
    column: usize,
}

impl<'a> Lexer<'a> {
    pub fn new(input: &'a str) -> Self {
        Self {
            _input: input,
            chars: input.chars().collect(),
            position: 0,
            line: 1,
            column: 1,
        }
    }

    fn peek(&self) -> Option<char> {
        if self.position < self.chars.len() {
            Some(self.chars[self.position])
        } else {
            None
        }
    }

    fn peek_next(&self) -> Option<char> {
        if self.position + 1 < self.chars.len() {
            Some(self.chars[self.position + 1])
        } else {
            None
        }
    }

    fn advance(&mut self) -> Option<char> {
        if self.position < self.chars.len() {
            let ch = self.chars[self.position];
            self.position += 1;
            if ch == '\n' {
                self.line += 1;
                self.column = 1;
            } else {
                self.column += 1;
            }
            Some(ch)
        } else {
            None
        }
    }

    pub fn tokenize(&mut self) -> Result<Vec<Token>, String> {
        let mut tokens = Vec::new();

        while let Some(ch) = self.peek() {
            if ch.is_whitespace() {
                self.advance();
                continue;
            }

            // Single line comments
            if ch == '/' && self.peek_next() == Some('/') {
                self.advance(); // consume '/'
                self.advance(); // consume '/'
                while let Some(c) = self.peek() {
                    if c == '\n' {
                        break;
                    }
                    self.advance();
                }
                continue;
            }

            let start_line = self.line;
            let start_col = self.column;

            // Numbers
            if ch.is_ascii_digit() {
                let mut num_str = String::new();
                while let Some(c) = self.peek() {
                    if c.is_ascii_digit() || c == '.' {
                        num_str.push(self.advance().unwrap());
                    } else {
                        break;
                    }
                }
                match num_str.parse::<f64>() {
                    Ok(val) => tokens.push(Token {
                        kind: TokenKind::Number(val),
                        line: start_line,
                        column: start_col,
                    }),
                    Err(_) => return Err(format!("Invalid number format '{}' at line {}, col {}", num_str, start_line, start_col)),
                }
                continue;
            }

            // Strings
            if ch == '"' {
                self.advance(); // consume opening quote
                let mut string_val = String::new();
                let mut closed = false;
                while let Some(c) = self.peek() {
                    if c == '"' {
                        self.advance();
                        closed = true;
                        break;
                    }
                    string_val.push(self.advance().unwrap());
                }
                if !closed {
                    return Err(format!("Unterminated string literal starting at line {}, col {}", start_line, start_col));
                }
                tokens.push(Token {
                    kind: TokenKind::String(string_val),
                    line: start_line,
                    column: start_col,
                });
                continue;
            }

            // Identifiers / Keywords
            if ch.is_alphabetic() || ch == '_' {
                let mut ident = String::new();
                while let Some(c) = self.peek() {
                    if c.is_alphanumeric() || c == '_' {
                        ident.push(self.advance().unwrap());
                    } else {
                        break;
                    }
                }
                let kind = match ident.as_str() {
                    "let" => TokenKind::Let,
                    "fn" => TokenKind::Fn,
                    "while" => TokenKind::While,
                    "if" => TokenKind::If,
                    "else" => TokenKind::Else,
                    "true" => TokenKind::True,
                    "false" => TokenKind::False,
                    _ => TokenKind::Identifier(ident),
                };
                tokens.push(Token {
                    kind,
                    line: start_line,
                    column: start_col,
                });
                continue;
            }

            // Operators & Punctuation
            let kind = match ch {
                '+' => { self.advance(); TokenKind::Plus }
                '-' => { self.advance(); TokenKind::Minus }
                '*' => { self.advance(); TokenKind::Asterisk }
                '/' => { self.advance(); TokenKind::Slash }
                '%' => { self.advance(); TokenKind::Percent }
                '(' => { self.advance(); TokenKind::LParen }
                ')' => { self.advance(); TokenKind::RParen }
                '{' => { self.advance(); TokenKind::LBrace }
                '}' => { self.advance(); TokenKind::RBrace }
                ',' => { self.advance(); TokenKind::Comma }
                ';' => { self.advance(); TokenKind::Semicolon }
                '=' => {
                    self.advance();
                    if self.peek() == Some('=') {
                        self.advance();
                        TokenKind::Eq
                    } else {
                        TokenKind::Assign
                    }
                }
                '!' => {
                    self.advance();
                    if self.peek() == Some('=') {
                        self.advance();
                        TokenKind::Ne
                    } else {
                        return Err(format!("Unexpected character '!' at line {}, col {}", start_line, start_col));
                    }
                }
                '<' => {
                    self.advance();
                    if self.peek() == Some('=') {
                        self.advance();
                        TokenKind::Le
                    } else {
                        TokenKind::Lt
                    }
                }
                '>' => {
                    self.advance();
                    if self.peek() == Some('=') {
                        self.advance();
                        TokenKind::Ge
                    } else {
                        TokenKind::Gt
                    }
                }
                '|' => {
                    self.advance();
                    if self.peek() == Some('>') {
                        self.advance();
                        TokenKind::Pipeline
                    } else {
                        return Err(format!("Unexpected character '|' at line {}, col {}", start_line, start_col));
                    }
                }
                _ => return Err(format!("Unexpected character '{}' at line {}, col {}", ch, start_line, start_col)),
            };

            tokens.push(Token {
                kind,
                line: start_line,
                column: start_col,
            });
        }

        tokens.push(Token {
            kind: TokenKind::EOF,
            line: self.line,
            column: self.column,
        });

        Ok(tokens)
    }
}
