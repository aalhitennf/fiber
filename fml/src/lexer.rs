use std::{path::Path, str::Chars};

pub type Result<I> = std::result::Result<I, Box<dyn std::error::Error>>;

#[derive(Default)]
pub struct Lexer<'a> {
    source: String,
    chars: Vec<char>,
    tokens: Vec<Token<'a>>,
    // cursor: usize,
}

#[derive(Debug)]
pub enum Token<'a> {
    TagOpen,
    TagClose,
    TagCloseSelf,
    Identifier(String),
    Equals,
    AttributeName(String),
    AttributeValue(String),
    Comment(String),
    Char(char),
    Error(LexerError),
    Unknown(&'a char),
    None,
}

#[derive(Debug)]
pub struct LexerError {
    message: String,
    source: String,
    line: usize,
    col: usize,
}

#[inline]
fn is_whitespace(c: &'static str) -> bool {
    c == " " || c == "\n" || c == "\t"
}

impl<'a> Lexer<'a> {
    pub fn new_from_path(p: impl AsRef<Path>) -> Result<Self> {
        let source = std::fs::read_to_string(p.as_ref())?;
        Self::new(&source)
    }

    pub fn new(s: &str) -> Result<Self> {
        let source = s.trim().to_string();
        let source_len = source.len();

        let lexer = Self {
            source,
            tokens: Vec::with_capacity(source_len),
            ..Lexer::default()
        };

        Ok(lexer)
    }

    pub fn lex(&mut self) {
        let mut line = 0;
        let mut col = 0;
        let mut cursor = 0;

        let mut opening_tag = false;
        let mut closing_tag = false;

        self.source = self.source.trim().to_string();

        if self.source.is_empty() {
            return;
        }

        let mut chars = self.source.chars();

        let advance_word = |chars: &mut Chars, cursor: &mut usize| {
            let mut buf = String::with_capacity(256);
            while let Some(ch) = chars.next() {
                *cursor += 1;

                if ch.is_alphanumeric() {
                    buf.push(ch);
                } else {
                    break;
                }
            }
            buf
        };

        // let read_to_tag_end = |chars: &mut Chars| {
        //     let mut buf = String::with_capacity(256);
        //     while let Some(ch) = chars.next() {
        //         if ch == '>' {
        //             break;
        //         } else {
        //             buf.push(ch);
        //         }
        //     }
        //     buf
        // };

        let read_tag_attributes = |cursor: &mut usize| -> &str {
            let mut lookahead = cursor.clone();

            loop {
                lookahead += 1;
                let next = self.chars.get(lookahead);

                if next.is_none() {
                    break;
                }

                if next.is_some_and(|ch| *ch == '>') {
                    break;
                }
            }

            *cursor = lookahead - 1;

            &self.source[*cursor..=lookahead]
        };

        while let Some(current) = chars.next() {
            if cursor == self.source.len() {
                break;
            }

            match current {
                '<' => {
                    self.tokens.push(Token::TagOpen);
                    let tag_name = advance_word(&mut chars, &mut cursor);
                    self.tokens.push(Token::Identifier(tag_name));
                    // let attrs = read_to_tag_end(&mut chars);
                    let attrs = read_tag_attributes(&mut cursor);
                    self.tokens.push(Token::AttributeName(attrs.to_string()));
                    // self.cursor = new_cursor;
                    // self.tokens.push(Token::TagClose);
                }

                '>' => {
                    self.tokens.push(Token::TagClose);
                }

                '\n' => {
                    line += 1;
                    col = 0;
                }

                _ => (),
            }

            cursor += 1;
            col += 1;
        }
    }
}

#[cfg(test)]
mod test {
    use super::Lexer;

    #[test]
    fn lex() {
        let mut lexer = Lexer::new_from_path("./test.fml").unwrap();
        lexer.lex();

        println!("{:#?}", lexer.tokens);
    }
}
