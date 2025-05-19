use core::fmt;
use std::process::exit;

#[derive(Debug, Clone, PartialEq)]
pub struct Span {
    pub start: usize,
    pub end: usize,
}
#[derive(Debug, Clone, PartialEq)]
pub enum TokenType {
    Ident(String),
    Number(String),
    StrLit(String),
    Operator(String),
    Separator(String),
    Keyword(String),
    Vardaic,

    Char(char),
    EOF,
    DOT,
    DCOLON,
    Comment(String),
}
#[derive(Debug, Clone, PartialEq)]
pub struct Token {
    pub kind: TokenType,
    pub span: Span,
}
impl fmt::Display for Token {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self.kind.clone() {
            TokenType::Ident(var) => {
                write!(f, "Identifier {var}")
            }
            TokenType::Number(num) => {
                write!(f, "Number {num}")
            }
            TokenType::StrLit(lit) => {
                write!(f, "String Constant: '{lit}'")
            }
            TokenType::Operator(opr) => {
                write!(f, "Binary Operator {opr}")
            }
            TokenType::Separator(sep) => {
                write!(f, "Separator '{sep}'")
            }
            TokenType::Keyword(word) => {
                write!(f, "Keyword '{word}'")
            }
            _ => {
                write!(f, "{:?}", self.kind.clone())
            }
        }
    }
}
pub struct Tokenizer {
    source: Vec<char>,
    pos: usize,
    start: usize,
    line: u64,
}

impl Tokenizer {
    pub fn new(input: &str) -> Self {
        Tokenizer {
            source: input.chars().collect(),
            pos: 0,
            start: 0,
            line: 1,
        }
    }

    fn peek(&self) -> Option<char> {
        self.source.get(self.pos).copied()
    }
    fn consume(&mut self) -> Option<char> {
        let ch = self.peek()?;
        if ch == '\n' {
            self.line += 1;
        }
        self.pos += 1;
        Some(ch)
    }
    fn skip_ws(&mut self) {
        while matches!(self.peek(), Some(c) if c.is_whitespace()) {
            self.consume();
        }
    }

    pub fn next_token(&mut self) -> Token {
        self.skip_ws();
        self.start = self.pos;
        match self.peek() {
            Some(c) if c.is_alphabetic() || c == '_' => self.identifier_or_keyword(),
            Some(c) if c.is_numeric() => self.number(),
            Some('"') => self.string_lit(),
            Some('\'') => {
                self.consume();
                if self.peek().unwrap() != '\'' {
                    let c = self.consume().unwrap();
                    if self.peek().unwrap() == '\'' {
                        self.consume();
                        return Token {
                            kind: TokenType::Char(c),
                            span: Span {
                                start: self.start,
                                end: self.pos,
                            },
                        };
                    } else {
                        println!("Expected closing \'");
                        exit(1)
                    }
                } else {
                    println!("Cannot have empty char literal");
                    exit(1);
                }
            }
            Some(c) if "+-*/=<>!&|%?".contains(c) => {
                if c == '/' {
                    if self.source.get(self.pos + 1).unwrap() == &'/' {
                        self.consume();
                        self.consume();
                        let mut comment = String::new();
                        while self.peek().unwrap() != '\n' {
                            comment.push(self.consume().unwrap());
                        }
                        return Token {
                            kind: TokenType::Comment(comment),
                            span: Span {
                                start: self.start,
                                end: self.pos,
                            },
                        };
                    } else if self.source.get(self.pos + 1).unwrap() == &'*' {
                        let mut comment = String::new();
                        self.consume();
                        self.consume();
                        while !(self.peek().unwrap() == '*'
                            && self.source.get(self.pos + 1).unwrap() == &'/')
                        {
                            comment.push(self.consume().unwrap());
                        }
                        self.consume();
                        self.consume();
                        return Token {
                            kind: TokenType::Comment(comment),
                            span: Span {
                                start: self.start,
                                end: self.pos,
                            },
                        };
                    }
                } else if c == '=' {
                    if self.source.get(self.pos + 1).unwrap() == &'=' {
                        self.consume();
                        self.consume();
                        return Token {
                            kind: TokenType::Operator("==".to_owned()),
                            span: Span {
                                start: self.start,
                                end: self.pos,
                            },
                        };
                    }
                } else if c == '!' {
                    if self.source.get(self.pos + 1).unwrap() == &'=' {
                        self.consume();
                        self.consume();
                        return Token {
                            kind: TokenType::Operator("!=".to_owned()),
                            span: Span {
                                start: self.start,
                                end: self.pos,
                            },
                        };
                    }
                } else if c == '<' {
                    if self.source.get(self.pos + 1).unwrap() == &'=' {
                        self.consume();
                        self.consume();
                        return Token {
                            kind: TokenType::Operator("<=".to_owned()),
                            span: Span {
                                start: self.start,
                                end: self.pos,
                            },
                        };
                    }
                } else if c == '>' {
                    if self.source.get(self.pos + 1).unwrap() == &'=' {
                        self.consume();
                        self.consume();
                        return Token {
                            kind: TokenType::Operator(">=".to_owned()),
                            span: Span {
                                start: self.start,
                                end: self.pos,
                            },
                        };
                    }
                }
                self.consume();
                Token {
                    kind: TokenType::Operator(format!("{c}")),
                    span: Span {
                        start: self.start,
                        end: self.pos,
                    },
                }
            }
            Some(c) if c == '.' => {
                self.consume();
                match self.peek().unwrap() == '.' {
                    true => {
                        self.consume();
                        match self.peek().unwrap() == '.' {
                            true => {
                                self.consume();
                                Token {
                                    kind: TokenType::Vardaic,
                                    span: Span {
                                        start: self.start,
                                        end: self.pos,
                                    },
                                }
                            }
                            false => {
                                println!("Unexpected character. missing '.' maybe?");
                                exit(100);
                            }
                        }
                    }
                    false => Token {
                        kind: TokenType::DOT,
                        span: Span {
                            start: self.start,
                            end: self.pos,
                        },
                    },
                }
            }
            Some(c) if "(){}[],;:".contains(c) => {
                if c == ':' {
                    self.consume();
                    if self.peek().unwrap() == ':' {
                        self.consume();
                        return Token {
                            kind: TokenType::DCOLON,
                            span: Span {
                                start: self.start,
                                end: self.pos,
                            },
                        };
                    }
                } else {
                    self.consume();
                }
                Token {
                    kind: TokenType::Separator(format!("{c}")),
                    span: Span {
                        start: self.start,
                        end: self.pos,
                    },
                }
            }
            Some(_) => {
                self.consume();
                Token {
                    kind: TokenType::EOF,
                    span: Span {
                        start: self.start,
                        end: self.pos,
                    },
                }
            }
            None => Token {
                kind: TokenType::EOF,
                span: Span {
                    start: self.start,
                    end: self.pos,
                },
            },
        }
    }
    fn identifier_or_keyword(&mut self) -> Token {
        self.start = self.pos;
        let mut ident = String::new();
        while let Some(c) = self.peek() {
            if c.is_alphanumeric() || c == '_' {
                ident.push(c);
                self.consume();
            } else {
                break;
            }
        }

        match ident.as_str() {
            "let" | "fn" | "if" | "else" | "while" | "ret" | "int" | "str" | "bool" | "buf"
            | "extern" | "i8" | "i16" | "i32" | "i64" | "u8" | "u16" | "u32" | "u64" | "char"
            | "struct" | "for" | "bundle" | "as" | "list" | "void" | "ptr" | "break" | "pack"
            | "unpack" | "with" | "continue" | "until" => Token {
                kind: TokenType::Keyword(ident),
                span: Span {
                    start: self.start,
                    end: self.pos,
                },
            },
            "JLINE" => Token {
                kind: TokenType::Number(format!("{}", self.line)),
                span: Span {
                    start: self.start,
                    end: self.pos,
                },
            },
            _ => Token {
                kind: TokenType::Ident(ident),
                span: Span {
                    start: self.start,
                    end: self.pos,
                },
            },
        }
    }
    fn number(&mut self) -> Token {
        self.start = self.pos;
        let mut number = String::new();
        while let Some(c) = self.peek() {
            if c.is_numeric() {
                number.push(c);
                self.consume();
            } else {
                break;
            };
        }

        Token {
            kind: TokenType::Number(number),
            span: Span {
                start: self.start,
                end: self.pos,
            },
        }
    }

    fn string_lit(&mut self) -> Token {
        self.start = self.pos;
        self.consume(); // opening "
        let mut content = String::new();
        while let Some(c) = self.peek() {
            if c == '"' {
                self.consume();
                break;
            }

            content.push(c);
            self.consume();
        }

        Token {
            kind: TokenType::StrLit(content),
            span: Span {
                start: self.start,
                end: self.pos,
            },
        }
    }
}
