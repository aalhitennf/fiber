#[derive(Debug, PartialEq)]
pub struct Token {
    pub kind: TokenKind,
    start: usize,
    end: usize,
}

#[derive(Debug, PartialEq)]
pub enum TokenKind {
    TagStart,     // <
    TagEnd,       // >
    TagClose,     // </
    TagSelfClose, // />
    TagName(String),
    AttributeName(String),
    AttributeValue(String),
    Equal,        // =
    Text(String), // Text content between tags
}

#[derive(Debug, Clone, PartialEq)]
pub enum AttributeValue {
    String(String),
    Float(f64),
    Int(i64),
}

pub struct Lexer {
    input: String,
    position: usize,
    line: usize,
    column: usize,
}

impl Lexer {
    pub fn new(input: String) -> Self {
        Lexer {
            input,
            position: 0,
            line: 1,
            column: 1,
        }
    }

    fn next_char(&mut self) -> Option<char> {
        if self.position < self.input.len() {
            let ch = self.input[self.position..].chars().next().unwrap();
            self.position += ch.len_utf8();
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

    fn peek_char(&self) -> Option<char> {
        if self.position < self.input.len() {
            self.input[self.position..].chars().next()
        } else {
            None
        }
    }

    fn skip_whitespace(&mut self) {
        while let Some(ch) = self.peek_char() {
            if ch.is_whitespace() {
                self.next_char();
            } else {
                break;
            }
        }
    }

    pub fn lex(&mut self) -> Vec<Token> {
        let mut tokens = Vec::new();
        let mut inside_tag = false;

        while let Some(ch) = self.next_char() {
            let start_pos = self.position - ch.len_utf8();
            match ch {
                '<' => {
                    inside_tag = true;
                    if let Some('/') = self.peek_char() {
                        self.next_char();
                        tokens.push(Token {
                            kind: TokenKind::TagClose,
                            start: start_pos,
                            end: self.position,
                        });
                    } else {
                        tokens.push(Token {
                            kind: TokenKind::TagStart,
                            start: start_pos,
                            end: self.position,
                        });
                    }
                }
                '>' => {
                    inside_tag = false;

                    tokens.push(Token {
                        kind: TokenKind::TagEnd,
                        start: start_pos,
                        end: self.position,
                    })
                }
                '/' => {
                    if let Some('>') = self.peek_char() {
                        self.next_char();
                        tokens.push(Token {
                            kind: TokenKind::TagSelfClose,
                            start: start_pos,
                            end: self.position,
                        });
                    }
                }
                '=' => tokens.push(Token {
                    kind: TokenKind::Equal,
                    start: start_pos,
                    end: self.position,
                }),
                '"' => {
                    let mut value = String::new();
                    let value_start_pos = self.position;
                    while let Some(next_ch) = self.next_char() {
                        if next_ch == '"' {
                            break;
                        }
                        value.push(next_ch);
                    }
                    tokens.push(Token {
                        kind: TokenKind::AttributeValue(value),
                        start: value_start_pos - 1, // Include the starting quote
                        end: self.position,
                    });
                }
                _ => {
                    if ch.is_alphabetic() {
                        // if inside_tag {
                        //     let mut name = String::new();
                        //     name.push(ch);
                        //     while let Some(next_ch) = self.peek_char() {
                        //         if next_ch.is_alphanumeric() || next_ch == '-' || next_ch == ':' {
                        //             name.push(self.next_char().unwrap());
                        //         } else {
                        //             break;
                        //         }
                        //     }
                        //     if let Some('=') = self.peek_char() {
                        //         tokens.push(Token::AttributeName(name));
                        //     } else {
                        //         tokens.push(Token::TagName(name));
                        //     }
                        // } else {
                        //     let mut text = String::new();
                        //     text.push(ch);
                        //     while let Some(next_ch) = self.peek_char() {
                        //         if next_ch == '<' {
                        //             break;
                        //         } else {
                        //             text.push(self.next_char().unwrap());
                        //         }
                        //     }
                        //     tokens.push(Token::Text(text));
                        // }
                        if inside_tag {
                            let mut name = String::new();
                            name.push(ch);
                            while let Some(next_ch) = self.peek_char() {
                                if next_ch.is_alphanumeric() || next_ch == '-' || next_ch == ':' {
                                    name.push(self.next_char().unwrap());
                                } else {
                                    break;
                                }
                            }
                            let end_pos = self.position;
                            if let Some('=') = self.peek_char() {
                                tokens.push(Token {
                                    kind: TokenKind::AttributeName(name),
                                    start: start_pos,
                                    end: end_pos,
                                });
                            } else {
                                tokens.push(Token {
                                    kind: TokenKind::TagName(name),
                                    start: start_pos,
                                    end: end_pos,
                                });
                            }
                        } else {
                            let mut text = String::new();
                            text.push(ch);
                            while let Some(next_ch) = self.peek_char() {
                                if next_ch == '<' {
                                    break;
                                } else {
                                    text.push(self.next_char().unwrap());
                                }
                            }
                            tokens.push(Token {
                                kind: TokenKind::Text(text),
                                start: start_pos,
                                end: self.position,
                            });
                        }
                    } else if ch.is_numeric() || ch == '.' || ch == '-' {
                        let mut value = String::new();
                        value.push(ch);
                        while let Some(next_ch) = self.peek_char() {
                            if next_ch.is_numeric() || next_ch == '.' {
                                value.push(self.next_char().unwrap());
                            } else {
                                break;
                            }
                        }
                        // if value.contains('.') {
                        //     if let Ok(f) = value.parse::<f64>() {
                        //         tokens.push(Token {
                        //             kind: TokenKind::AttributeValue(AttributeValue::Float(f),
                        //             start: start_pos,
                        //             end: self.position,
                        //         });
                        //     } else {
                        //         tokens.push(Token {
                        //             kind: TokenKind::Error(format!(
                        //                 "{value} cannot be parser as f64"
                        //             )),
                        //             start: start_pos,
                        //             end: self.position,
                        //         });
                        //     }
                        // } else {
                        //     if let Ok(i) = value.parse::<i64>() {
                        //         tokens.push(Token::AttributeValue(AttributeValue::Int(i)));
                        //     } else {
                        //         tokens
                        //             .push(Token::Error(format!("{value} cannot be parser as i64")));
                        //     }
                        // };

                        tokens.push(Token {
                            kind: TokenKind::AttributeValue(value),
                            start: start_pos,
                            end: self.position,
                        });
                    } else if !ch.is_whitespace() {
                        let mut text = String::new();
                        text.push(ch);
                        while let Some(next_ch) = self.peek_char() {
                            if next_ch == '<' {
                                break;
                            } else {
                                text.push(self.next_char().unwrap());
                            }
                        }
                        tokens.push(Token {
                            kind: TokenKind::Text(text),
                            start: start_pos,
                            end: self.position,
                        });
                    }
                }
            }
            self.skip_whitespace();
        }

        tokens
    }
}

#[test]
fn lex() {
    let content = std::fs::read_to_string("./test.fml").unwrap();
    let mut lexer = Lexer::new(content);
    let tokens = lexer.lex();

    println!("{:#?}", tokens);
}
// }
