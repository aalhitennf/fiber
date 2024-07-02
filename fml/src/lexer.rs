use std::fmt::Display;

#[derive(Debug)]
pub struct Token<'a> {
    pub kind: TokenKind<'a>,
    pub start: usize,
    pub end: usize,
    pub line: usize,
    pub col: usize,
}

#[derive(Debug, PartialEq)]
pub enum TokenKind<'a> {
    TagStart,     // <
    TagEnd,       // >
    TagClose,     // </
    TagSelfClose, // />
    TagName(&'a str),
    AttributeName(&'a str),
    AttributeValue(&'a str),
    Variable(&'a str),
    EqualSign,     // =
    Text(&'a str), // Text content between tags
    LineComment(&'a str),
}

impl<'a> Display for TokenKind<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            TokenKind::TagStart => write!(f, "<"),
            TokenKind::TagEnd => write!(f, ">"),
            TokenKind::TagClose => write!(f, "</"),
            TokenKind::TagSelfClose => write!(f, "/>"),
            TokenKind::TagName(name) => write!(f, "TagName: {name}"),
            TokenKind::AttributeName(name) => write!(f, "AttributeName: {name}"),
            TokenKind::AttributeValue(value) => write!(f, "AttributeValue: {value}"),
            TokenKind::Variable(name) => write!(f, "Variable: {name}"),
            TokenKind::EqualSign => write!(f, "="),
            TokenKind::Text(text) => write!(f, "Text content between tags: {text}"),
            TokenKind::LineComment(comment) => write!(f, "LineComment: {comment}"),
        }
    }
}

pub struct Lexer<'a> {
    input: &'a str,
    position: usize,
    line: usize,
    column: usize,
}

impl<'a> Lexer<'a> {
    #[must_use]
    pub fn new(input: &'a str) -> Self {
        Lexer {
            input,
            position: 0,
            line: 1,
            column: 0,
        }
    }

    #[inline]
    fn next_char(&mut self) -> Option<char> {
        let ch = self.input[self.position..].chars().next()?;

        self.position += ch.len_utf8();

        if ch == '\n' {
            self.line += 1;
            self.column = 0;
        } else {
            self.column += 1;
        }
        Some(ch)
    }

    #[inline]
    fn peek_char(&self) -> Option<char> {
        self.input[self.position..].chars().next()
    }

    #[inline]
    fn skip_whitespace(&mut self) {
        while let Some(ch) = self.peek_char() {
            if ch.is_whitespace() {
                self.next_char();
            } else {
                break;
            }
        }
    }

    #[allow(clippy::too_many_lines)]
    pub fn lex(&mut self) -> Vec<Token<'a>> {
        let mut tokens = Vec::with_capacity(self.input.len());
        let mut inside_tag = false;
        let mut start_pos;
        let mut value_start_pos;

        while let Some(ch) = self.next_char() {
            start_pos = self.position - ch.len_utf8();

            match ch {
                '<' => {
                    inside_tag = true;

                    if let Some('/') = self.peek_char() {
                        self.next_char();

                        tokens.push(Token {
                            kind: TokenKind::TagClose,
                            start: start_pos,
                            end: self.position,
                            line: self.line,
                            col: self.column,
                        });
                    } else {
                        tokens.push(Token {
                            kind: TokenKind::TagStart,
                            start: start_pos,
                            end: self.position,
                            line: self.line,
                            col: self.column,
                        });
                    }
                }
                '>' => {
                    inside_tag = false;

                    tokens.push(Token {
                        kind: TokenKind::TagEnd,
                        start: start_pos,
                        end: self.position,
                        line: self.line,
                        col: self.column,
                    });
                }
                '/' => match self.peek_char() {
                    Some('>') => {
                        self.next_char();

                        tokens.push(Token {
                            kind: TokenKind::TagSelfClose,
                            start: start_pos,
                            end: self.position,
                            line: self.line,
                            col: self.column,
                        });
                    }

                    Some('/') => {
                        while let Some(next_ch) = self.peek_char() {
                            if next_ch == '\n' {
                                tokens.push(Token {
                                    kind: TokenKind::LineComment(&self.input[start_pos..self.position]),
                                    start: start_pos,
                                    end: self.position,
                                    line: self.line,
                                    col: self.column,
                                });
                                break;
                            }

                            self.next_char();
                        }
                    }

                    None => break,
                    _ => (),
                },
                '=' if inside_tag => tokens.push(Token {
                    kind: TokenKind::EqualSign,
                    start: start_pos,
                    end: self.position,
                    line: self.line,
                    col: self.column,
                }),

                '{' => {
                    while let Some(next_ch) = self.next_char() {
                        if next_ch == '}' {
                            break;
                        }
                    }

                    tokens.push(Token {
                        kind: TokenKind::Variable(&self.input[(start_pos + 1)..(self.position - 1)]),
                        start: start_pos,
                        end: self.position,
                        line: self.line,
                        col: self.column,
                    })
                }

                '"' if inside_tag => {
                    value_start_pos = self.position;

                    while let Some(next_ch) = self.next_char() {
                        if next_ch == '"' {
                            break;
                        }
                    }

                    tokens.push(Token {
                        kind: TokenKind::AttributeValue(&self.input[start_pos..self.position]),
                        start: value_start_pos - 1,
                        end: self.position,
                        line: self.line,
                        col: self.column,
                    });
                }

                '\n' | '\t' => (),

                _ => {
                    if inside_tag {
                        if ch.is_alphabetic() || ch == '_' || ch == '-' {
                            while let Some(next_ch) = self.peek_char() {
                                if next_ch.is_alphanumeric() || next_ch == '-' || next_ch == ':' || next_ch == '_' {
                                    self.next_char();
                                } else {
                                    break;
                                }
                            }

                            let end_pos = self.position;

                            if let Some('=') = self.peek_char() {
                                tokens.push(Token {
                                    kind: TokenKind::AttributeName(&self.input[start_pos..self.position]),
                                    start: start_pos,
                                    end: end_pos,
                                    line: self.line,
                                    col: self.column,
                                });
                            } else {
                                tokens.push(Token {
                                    kind: TokenKind::TagName(&self.input[start_pos..self.position]),
                                    start: start_pos,
                                    end: end_pos,
                                    line: self.line,
                                    col: self.column,
                                });
                            }
                        } else if ch.is_numeric() || ch == '.' || ch == '-' {
                            while let Some(next_ch) = self.peek_char() {
                                if next_ch.is_numeric() || next_ch == '.' {
                                    self.next_char();
                                } else {
                                    break;
                                }
                            }

                            tokens.push(Token {
                                kind: TokenKind::AttributeValue(&self.input[start_pos..self.position]),
                                start: start_pos,
                                end: self.position,
                                line: self.line,
                                col: self.column,
                            });
                        } else if !ch.is_whitespace() {
                            while let Some('<') = self.peek_char() {
                                self.next_char();
                            }

                            tokens.push(Token {
                                kind: TokenKind::Text(&self.input[start_pos..self.position]),
                                start: start_pos,
                                end: self.position,
                                line: self.line,
                                col: self.column,
                            });
                        }
                    } else {
                        while let Some(next_ch) = self.peek_char() {
                            if next_ch == '<' {
                                break;
                            }

                            self.next_char();
                        }

                        tokens.push(Token {
                            kind: TokenKind::Text(&self.input[start_pos..self.position]),
                            start: start_pos,
                            end: self.position,
                            line: self.line,
                            col: self.column,
                        });
                    }
                }
            }
            self.skip_whitespace();
        }

        tokens
    }
}
