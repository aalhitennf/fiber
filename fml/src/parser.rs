use crate::lexer::{Token, TokenKind};

#[derive(Debug)]
pub struct Attribute<'a> {
    pub name: &'a str,
    pub value: &'a str,
}

#[derive(Debug)]
pub enum Node<'a> {
    Element(Element<'a>),
    Text(&'a str),
}

#[derive(Debug)]
pub struct Element<'a> {
    pub name: &'a str,
    pub attributes: Vec<Attribute<'a>>,
    pub children: Vec<Node<'a>>,
}

pub struct Parser<'a> {
    tokens: Vec<Token<'a>>,
    position: usize,
}

impl<'a> Parser<'a> {
    #[must_use]
    pub fn new(tokens: Vec<Token<'a>>) -> Self {
        Parser { tokens, position: 0 }
    }

    fn current_token(&self) -> Option<&Token<'a>> {
        self.tokens.get(self.position)
    }

    fn advance(&mut self) {
        self.position += 1;
    }

    fn parse_attributes(&mut self) -> Result<Vec<Attribute<'a>>, String> {
        let mut attributes = Vec::new();

        while let Some(token) = self.current_token() {
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

                    attributes.push(Attribute { name: attr_name, value });
                }
                _ => break,
            }
        }

        Ok(attributes)
    }

    fn parse_children(&mut self, parent_name: &'a str) -> Result<Vec<Node<'a>>, String> {
        let mut children = Vec::new();

        loop {
            match self.current_token() {
                Some(Token {
                    kind: TokenKind::TagStart,
                    ..
                }) => {
                    if let Some(Token {
                        kind: TokenKind::TagClose,
                        ..
                    }) = self.tokens.get(self.position + 1)
                    {
                        break;
                    }
                    children.push(Node::Element(self.parse_element()?));
                }
                Some(Token {
                    kind: TokenKind::Text(text),
                    ..
                }) => {
                    children.push(Node::Text(text));
                    self.advance();
                }
                _ => break,
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
        // let name = if let Some(Token {
        //     kind: TokenKind::TagName(name),
        //     ..
        // }) = self.current_token()
        // {
        //     name
        // } else {
        //     return Err("Expected TagName".to_string());
        // };

        let name = {
            let token = self.current_token().ok_or("Expected TagName")?;
            if let TokenKind::TagName(name) = &token.kind {
                name.clone()
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
            return Ok(Element {
                name,
                attributes,
                children: Vec::new(),
            });
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

        let children = self.parse_children(name)?;

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
            if *close_name != name {
                return Err(format!(
                    "Mismatched closing tag: expected {}, found {}",
                    name, close_name
                ));
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

        Ok(Element {
            name,
            attributes,
            children,
        })
    }

    pub fn parse(&mut self) -> Result<Vec<Node<'a>>, String> {
        let mut nodes = Vec::new();

        while let Ok(element) = self.parse_element() {
            nodes.push(Node::Element(element));
        }

        // while self.current_token().is_some() {
        //     if let Some(Token {
        //         kind: TokenKind::TagStart,
        //         ..
        //     }) = self.current_token()
        //     {
        //         nodes.push(Node::Element(self.parse_element()?));
        //     } else {
        //         self.advance();
        //     }
        // }

        Ok(nodes)
    }
}
