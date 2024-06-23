use std::{path::Path, rc::Rc};

pub type Result<I> = std::result::Result<I, Box<dyn std::error::Error>>;

pub struct Lexer<'a> {
    source: Rc<String>,
    tokens: Vec<Token<'a>>,
    cursor: Cursor,
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
        let source = Rc::new(s.trim().to_string());
        let source_len = source.len();
        let cursor = Cursor::new(source.clone());

        let lexer = Self {
            source,
            tokens: Vec::with_capacity(source_len),
            cursor,
        };

        Ok(lexer)
    }

    pub fn lex(&mut self) {
        while let Some(ch) = self.cursor.next() {
            match ch {
                '<' => {
                    self.tokens.push(Token::TagOpen);

                    let tag_name = self.cursor.advance_word();
                    self.tokens.push(Token::Identifier(tag_name));

                    let attrs = self.cursor.read_tag_definition();
                    self.tokens.push(Token::AttributeName(attrs.to_string()));

                    self.tokens.push(Token::TagClose);
                }

                '>' => {
                    self.tokens.push(Token::TagClose);
                }

                '\n' => {
                    // line += 1;
                    // col = 0;
                }

                other => (),
                // other => self.tokens.push(Token::Char(other)),
            }
        }
    }
}

#[derive(Default)]
pub struct Cursor {
    source: Rc<String>,
    pos: usize,
}

impl Cursor {
    pub fn new(source: Rc<String>) -> Self {
        Self { source, pos: 0 }
    }
}

impl Iterator for Cursor {
    type Item = char;

    fn next(&mut self) -> Option<Self::Item> {
        let item = self.source.as_bytes().get(self.pos).map(|u| *u as char);
        self.pos += 1;
        item
    }
}

impl Cursor {
    pub fn peek_next(&self) -> Option<char> {
        self.source.as_bytes().get(self.pos + 1).map(|u| *u as char)
    }

    pub fn peek_previous(&self) -> Option<char> {
        if self.pos == 0 || self.source.is_empty() {
            return None;
        }

        self.source.as_bytes().get(self.pos - 1).map(|u| *u as char)
    }

    pub fn step_back(&mut self) {
        self.pos = self.pos.saturating_sub(1);
    }

    pub fn step_forward(&mut self) {
        self.pos = self.pos.saturating_add(1);
    }

    pub fn advance_word(&mut self) -> String {
        let start = self.pos;

        while let Some(_) = self.next() {
            if self.peek_next().is_some_and(|c| !c.is_alphanumeric()) || self.peek_next().is_none()
            {
                break;
            }
        }

        self.step_forward();

        self.source
            .get(start..=self.pos)
            .map_or_else(|| String::new(), |slice| slice.to_string())
    }

    pub fn read_tag_definition(&mut self) -> String {
        let start = self.pos;

        while let Some(ch) = self.next() {
            if ch == '>' {
                break;
            }
        }

        self.source
            .get(start..self.pos)
            .map_or_else(|| String::new(), |slice| slice.to_string())
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
