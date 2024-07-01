mod analyzer;
mod attr;
mod element;
mod error;

use std::borrow::Cow;

pub use attr::{Attribute, AttributeValue};
pub use element::{Element, ElementKind, Node};

use crate::lexer::{Token, TokenKind};

pub struct Parser<'a> {
    tokens: Vec<Token<'a>>,
    position: usize,
}

impl<'a> Parser<'a> {
    #[must_use]
    pub fn new(mut tokens: Vec<Token<'a>>) -> Self {
        tokens.retain(|t| {
            !matches!(
                t,
                Token {
                    kind: TokenKind::LineComment(_),
                    ..
                }
            )
        });
        Parser { tokens, position: 0 }
    }

    #[inline]
    fn current_token(&self) -> Option<&Token<'a>> {
        self.tokens.get(self.position)
    }

    #[inline]
    fn advance(&mut self) {
        self.position += 1;
    }

    #[inline]
    fn parse_attributes(&mut self) -> Result<Vec<Attribute<'a>>, String> {
        let mut attributes = Vec::new();
        let mut line;
        let mut col;

        while let Some(token) = self.current_token().as_ref() {
            line = token.line;
            col = token.col;

            match token.kind {
                TokenKind::AttributeName(attr_name) => {
                    self.advance();

                    if !matches!(
                        self.current_token(),
                        Some(Token {
                            kind: TokenKind::EqualSign,
                            ..
                        })
                    ) {
                        return Err("Expected Equal (=)".to_string());
                    }
                    self.advance();

                    let value = if let Some(token) = self.current_token() {
                        if let TokenKind::AttributeValue(attr_value) = token.kind {
                            attr_value
                        } else {
                            return Err("Expected AttributeValue".to_string());
                        }
                    } else {
                        return Err("Expected AttributeValue".to_string());
                    };
                    self.advance();

                    attributes.push(Attribute {
                        name: Cow::Borrowed(attr_name),
                        value: AttributeValue::new(value, line, col)?,
                    });
                }
                _ => break,
            }
        }

        Ok(attributes)
    }

    #[inline]
    fn parse_children(&mut self) -> Result<Vec<Node<'a>>, String> {
        let mut children = Vec::with_capacity(20);

        loop {
            if let Some(token) = self.current_token() {
                match token.kind {
                    TokenKind::TagStart => {
                        if let Some(Token {
                            kind: TokenKind::TagClose,
                            ..
                        }) = self.tokens.get(self.position + 1)
                        {
                            break;
                        }
                        children.push(Node::Element(self.parse_element()?));
                    }
                    TokenKind::Text(text) => {
                        children.push(Node::Text(text));
                        self.advance();
                    }
                    _ => break,
                }
            }
        }

        Ok(children)
    }

    #[allow(clippy::too_many_lines)]
    fn parse_element(&mut self) -> Result<Element<'a>, String> {
        {
            if !matches!(
                self.current_token(),
                Some(Token {
                    kind: TokenKind::TagStart,
                    ..
                })
            ) {
                return Err("Expected TagStart".to_string());
            }
        }

        self.advance();

        let name = {
            let token = self.current_token().ok_or("Expected TagName")?;
            if let TokenKind::TagName(name) = token.kind {
                name
            } else {
                return Err("Expected TagName".to_string());
            }
        };

        self.advance();

        let attributes = self.parse_attributes()?;

        if let Some(Token {
            kind: TokenKind::TagSelfClose,
            ..
        }) = self.current_token()
        {
            self.advance();

            return Ok(Element::new(name, attributes, Vec::new()));
        }

        if !matches!(
            self.current_token(),
            Some(Token {
                kind: TokenKind::TagEnd,
                ..
            })
        ) {
            return Err("Expected TagEnd".to_string());
        }

        self.advance();

        let children = self.parse_children()?;

        if !matches!(
            self.current_token(),
            Some(Token {
                kind: TokenKind::TagClose,
                ..
            })
        ) {
            return Err("Expected TagClose".to_string());
        }

        self.advance();

        if let Some(Token {
            kind: TokenKind::TagName(close_name),
            ..
        }) = self.current_token()
        {
            if close_name != &name {
                return Err(format!("Mismatched closing tag: expected {name}, found {close_name}"));
            }
        } else {
            return Err("Expected TagName".to_string());
        }

        self.advance();

        if !matches!(
            self.current_token(),
            Some(Token {
                kind: TokenKind::TagEnd,
                ..
            })
        ) {
            return Err("Expected TagEnd".to_string());
        }

        self.advance();

        Ok(Element::new(name, attributes, children))
    }

    #[allow(clippy::missing_errors_doc)]
    pub fn parse(&mut self) -> Result<Vec<Node<'a>>, String> {
        let mut nodes = Vec::new();

        while let Ok(element) = self.parse_element() {
            nodes.push(Node::Element(element));
        }

        Ok(nodes)
    }
}
