use crate::lexer::{Token, TokenKind};

#[derive(Debug)]
pub struct Attribute<'a> {
    pub name: &'a str,
    pub value: AttributeValue<'a>,
}

#[derive(Debug)]
pub enum AttributeValue<'a> {
    String(&'a str),
    Integer(i64),
    Float(f64),
}

impl<'a> From<&'a str> for AttributeValue<'a> {
    #[inline]
    fn from(input: &'a str) -> AttributeValue {
        if let Ok(i) = input.parse::<i64>() {
            return AttributeValue::Integer(i);
        }

        if let Ok(f) = input.parse::<f64>() {
            return AttributeValue::Float(f);
        }

        AttributeValue::String(input)
    }
}

#[derive(Debug)]
pub enum Node<'a> {
    Element(Element<'a>),
    Text(&'a str),
}

#[derive(Debug)]
pub struct Element<'a> {
    pub kind: ElementKind<'a>,
    pub attributes: Vec<Attribute<'a>>,
    pub children: Vec<Node<'a>>,
}

#[derive(Debug)]
pub enum ElementKind<'a> {
    Root,
    Box,
    VStack,
    HStack,
    Clip,
    Label,
    Button,
    Input,
    Image,
    Empty,
    Custom(&'a str),
}

impl<'a> Element<'a> {
    #[must_use]
    pub fn new(name: &'a str, attributes: Vec<Attribute<'a>>, children: Vec<Node<'a>>) -> Element<'a> {
        let kind = match name {
            "root" => ElementKind::Root,
            "box" => ElementKind::Box,
            "vstack" => ElementKind::VStack,
            "hstack" => ElementKind::HStack,
            "clip" => ElementKind::Clip,
            "label" => ElementKind::Label,
            "button" => ElementKind::Button,
            "input" => ElementKind::Input,
            "image" => ElementKind::Image,
            "" => ElementKind::Empty,
            other => ElementKind::Custom(other),
        };

        Element {
            kind,
            attributes,
            children,
        }
    }
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

                    attributes.push(Attribute {
                        name: attr_name,
                        value: AttributeValue::from(value),
                    });
                }
                _ => break,
            }
        }

        Ok(attributes)
    }

    #[inline]
    fn parse_children(&mut self) -> Result<Vec<Node<'a>>, String> {
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
