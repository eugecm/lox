use std::{
    fmt::{Display, Write},
    iter::Peekable,
    str::Chars,
};

use thiserror::Error;

#[derive(Error, Debug)]
pub enum ParserError {
    #[error("[line {line}] Error {loc}: {msg}")]
    UnexpectedToken {
        line: usize,
        loc: String,
        msg: String,
    },
    #[error("[line {line}] Error {loc}: {msg}")]
    UnterminatedString {
        line: usize,
        loc: String,
        msg: String,
    },
}

pub struct Scanner<'a> {
    contents: &'a str,
}

impl<'a> Scanner<'a> {
    pub fn new(c: &'a str) -> Self {
        Self { contents: c }
    }

    pub fn scan_tokens(&self) -> Tokens {
        Tokens {
            contents: self.contents,
            chars: self.contents.chars().peekable(),
            cursor: 0,
            line: 0,
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct Token<'a> {
    pub typ: TokenType,
    pub lexeme: &'a str,
    pub line: usize,
}

impl<'a> Token<'a> {
    pub fn new(typ: TokenType, lexeme: &'a str, line: usize) -> Self {
        Self { typ, lexeme, line }
    }
}

impl Display for Token<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.lexeme)
    }
}

pub struct Tokens<'a> {
    // Represents the raw content that we're parsing
    contents: &'a str,
    // The Unicode characters points that we're parsing
    chars: Peekable<Chars<'a>>,
    cursor: usize,
    line: usize,
}

impl<'a> Tokens<'a> {
    fn is_at_end(&mut self) -> bool {
        self.chars.peek().is_none()
    }

    fn advance(&mut self) -> Option<char> {
        self.chars.next().inspect(|c| self.cursor += c.len_utf8())
    }

    fn matches(&mut self, c: char) -> bool {
        if self.chars.peek() == Some(&c) {
            self.advance();
            return true;
        }

        return false;
    }

    fn next_if<F>(&mut self, f: F) -> Option<char>
    where
        F: FnOnce(&char) -> bool,
    {
        self.chars
            .next_if(f)
            .inspect(|c| self.cursor += c.len_utf8())
    }
}

impl<'a> Iterator for Tokens<'a> {
    type Item = Result<Token<'a>, ParserError>;

    fn next(&mut self) -> Option<Self::Item> {
        // scanToken in the book
        let mut literal_start = self.cursor;
        let (lexeme, token_type) = loop {
            let lexeme = self.advance()?;
            let token_type = match lexeme {
                // Simple cases
                '(' => Some(TokenType::LeftParen),
                ')' => Some(TokenType::RightParen),
                '{' => Some(TokenType::LeftBrace),
                '}' => Some(TokenType::RightBrace),
                ',' => Some(TokenType::Comma),
                '.' => Some(TokenType::Dot),
                '-' => Some(TokenType::Minus),
                '+' => Some(TokenType::Plus),
                ';' => Some(TokenType::Semicolon),
                '*' => Some(TokenType::Star),

                // More complex cases
                '!' => {
                    if self.matches('=') {
                        Some(TokenType::BangEqual)
                    } else {
                        Some(TokenType::Bang)
                    }
                }
                '=' => {
                    if self.matches('=') {
                        Some(TokenType::EqualEqual)
                    } else {
                        Some(TokenType::Equal)
                    }
                }
                '<' => {
                    if self.matches('=') {
                        Some(TokenType::LessEqual)
                    } else {
                        Some(TokenType::Less)
                    }
                }
                '>' => {
                    if self.matches('=') {
                        Some(TokenType::GreaterEqual)
                    } else {
                        Some(TokenType::Greater)
                    }
                }
                '/' => {
                    if self.matches('/') {
                        // This is a comment
                        while self.next_if(|c| *c != '\n').is_some() {}
                        // We either reached the \n or the EOF
                        Some(TokenType::Comment)
                    } else {
                        Some(TokenType::Slash)
                    }
                }

                '"' => {
                    while let Some(c) = self.next_if(|c| *c != '"') {
                        if c == '\n' {
                            self.line += 1;
                        }
                    }
                    if self.advance().is_none() {
                        return Some(Err(ParserError::UnterminatedString {
                            line: self.line,
                            loc: self.contents[literal_start..self.cursor].to_string(),
                            msg: "Unterminated string".to_string(),
                        }));
                    };
                    Some(TokenType::String)
                }

                // Ignore whitespaces
                ' ' | '\r' | 't' => {
                    literal_start = self.cursor;
                    None
                }
                '\n' => {
                    literal_start = self.cursor;
                    self.line += 1;
                    None
                }

                // Unexpected
                c => {
                    if c.is_digit(10) {
                        // Parse number
                        while self.next_if(|c| c.is_digit(10)).is_some() {}
                        if let Some('.') = self.chars.peek() {
                            // Check if the char afterwards is some digit
                            let after_dot = self.contents[self.cursor + 1..].chars().next();
                            if let Some(p) = after_dot {
                                if p.is_digit(10) {
                                    self.advance(); // consume the dot
                                    while self.next_if(|c| c.is_digit(10)).is_some() {}
                                }
                            }
                        }
                        Some(TokenType::Number)
                    } else if c.is_alphabetic() {
                        while self.next_if(|c| c.is_alphanumeric()).is_some() {}

                        if let Some(reserved) =
                            try_reserved(&self.contents[literal_start..self.cursor])
                        {
                            Some(reserved)
                        } else {
                            Some(TokenType::Identifier)
                        }
                    } else {
                        return Some(Err(ParserError::UnexpectedToken {
                            line: self.line,
                            loc: self.contents[literal_start..self.cursor].to_string(),
                            msg: "Unexpected token".to_string(),
                        }));
                    }
                }
            };

            if let Some(token_type) = token_type {
                break (lexeme, token_type);
            } else {
                continue;
            }
        };

        Some(Ok(Token {
            typ: token_type,
            lexeme: if token_type == TokenType::String {
                &self.contents[literal_start + 1..self.cursor - 1]
            } else {
                &self.contents[literal_start..self.cursor]
            },
            line: self.line,
        }))
    }
}

pub fn try_reserved(word: &str) -> Option<TokenType> {
    match word {
        "and" => Some(TokenType::And),
        "class" => Some(TokenType::Class),
        "else" => Some(TokenType::Else),
        "false" => Some(TokenType::False),
        "for" => Some(TokenType::For),
        "fun" => Some(TokenType::Fun),
        "if" => Some(TokenType::If),
        "nil" => Some(TokenType::Nil),
        "or" => Some(TokenType::Or),
        "print" => Some(TokenType::Print),
        "return" => Some(TokenType::Return),
        "super" => Some(TokenType::Super),
        "this" => Some(TokenType::This),
        "true" => Some(TokenType::True),
        "var" => Some(TokenType::Var),
        "while" => Some(TokenType::While),
        _ => None,
    }
}

#[derive(Debug, PartialEq, Eq, Copy, Clone)]
pub enum TokenType {
    // Single-character tokens.
    LeftParen,
    RightParen,
    LeftBrace,
    RightBrace,
    Comma,
    Dot,
    Minus,
    Plus,
    Semicolon,
    Slash,
    Star,

    // One or two character tokens.
    Bang,
    BangEqual,
    Equal,
    EqualEqual,
    Greater,
    GreaterEqual,
    Less,
    LessEqual,
    Comment,

    // Literals.
    Identifier,
    String,
    Number,

    // Keywords.
    And,
    Class,
    Else,
    False,
    Fun,
    For,
    If,
    Nil,
    Or,
    Print,
    Return,
    Super,
    This,
    True,
    Var,
    While,

    Eof,
}
