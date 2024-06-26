use std::fmt::Display;

use crate::lexer::{Token, TokenKind};

#[derive(Debug)]
pub struct Attribute<'a> {
    pub name: &'a str,
    pub value: Option<&'a str>,
}

#[derive(Debug)]
pub struct Element<'a> {
    pub name: &'a str,
    pub attributes: Vec<Attribute<'a>>,
    pub children: Vec<Node<'a>>,
}

#[derive(Debug)]
pub enum Node<'a> {
    Element(Element<'a>),
    Text(&'a str),
}

#[derive(Debug)]
pub enum ParseError<'a> {
    UnexpectedToken {
        expected: TokenKind<'a>,
        found: Option<TokenKind<'a>>,
        position: usize,
    },
    UnexpectedEOF,
}

impl<'a> Display for ParseError<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ParseError::UnexpectedToken {
                expected,
                found,
                position,
            } => write!(
                f,
                "Error at position {}: expected {}, found {:?}",
                position, expected, found
            ),
            ParseError::UnexpectedEOF => write!(f, "Error: unexpected end of file"),
        }
    }
}

pub struct Parser<'a> {
    tokens: Vec<Token<'a>>,
    position: usize,
}

impl<'a> Parser<'a> {
    pub fn new(tokens: Vec<Token<'a>>) -> Self {
        Parser { tokens, position: 0 }
    }

    fn current_token(&self) -> Option<&Token<'a>> {
        self.tokens.get(self.position)
    }

    fn next_token(&mut self) -> Option<&Token<'a>> {
        self.position += 1;
        self.tokens.get(self.position)
    }

    fn expect_token(&self, expected: TokenKind<'a>) -> Result<&Token<'a>, ParseError<'a>> {
        match self.current_token() {
            Some(token) if token.kind == expected => Ok(token),
            Some(token) => Err(ParseError::UnexpectedToken {
                expected: expected,
                found: Some(token.kind.clone()),
                position: self.position,
            }),
            None => Err(ParseError::UnexpectedEOF),
        }
    }

    fn parse_attributes(&mut self) -> Result<Vec<Attribute<'a>>, ParseError<'a>> {
        let mut attributes = Vec::new();

        while let Some(token) = self.current_token() {
            match &token.kind {
                TokenKind::AttributeName(name) => {
                    let attr_name = *name;
                    self.next_token(); // consume AttributeName
                    let attr_value = if matches!(self.current_token().map(|t| &t.kind), Some(TokenKind::EqualSign)) {
                        self.next_token(); // consume EqualSign
                        if let Some(TokenKind::AttributeValue(value)) = self.next_token().map(|t| &t.kind) {
                            Some(*value)
                        } else {
                            return Err(ParseError::UnexpectedToken {
                                expected: TokenKind::AttributeValue("value"),
                                found: self.current_token().map(|t| t.kind.clone()),
                                position: self.position,
                            });
                        }
                    } else {
                        None
                    };
                    attributes.push(Attribute {
                        name: attr_name,
                        value: attr_value,
                    });
                }
                _ => break,
            }
        }

        Ok(attributes)
    }

    fn parse_element(&mut self) -> Result<Element<'a>, ParseError<'a>> {
        self.expect_token(TokenKind::TagStart)?;
        self.next_token(); // consume TagStart

        let name = match self.current_token().map(|t| &t.kind) {
            Some(TokenKind::TagName(name)) => *name,
            found => {
                return Err(ParseError::UnexpectedToken {
                    expected: TokenKind::TagName("name"),
                    found: found.cloned(),
                    position: self.position,
                });
            }
        };
        self.next_token(); // consume TagName

        let attributes = self.parse_attributes()?;

        if matches!(self.current_token().map(|t| &t.kind), Some(TokenKind::TagSelfClose)) {
            self.next_token(); // consume TagSelfClose
            return Ok(Element {
                name,
                attributes,
                children: Vec::new(),
            });
        }

        self.expect_token(TokenKind::TagEnd)?;
        self.next_token(); // consume TagEnd

        let mut children = Vec::new();
        while !matches!(self.current_token().map(|t| &t.kind), Some(TokenKind::TagClose)) {
            if let Some(child) = self.parse_node()? {
                children.push(child);
            } else {
                break;
            }
        }

        self.expect_token(TokenKind::TagClose)?;
        self.next_token(); // consume TagClose

        match self.current_token().map(|t| &t.kind) {
            Some(TokenKind::TagName(close_name)) if *close_name == name => {
                self.next_token(); // consume close TagName
            }
            found => {
                return Err(ParseError::UnexpectedToken {
                    expected: TokenKind::TagName(name),
                    found: found.cloned(),
                    position: self.position,
                });
            }
        }

        self.expect_token(TokenKind::TagEnd)?;
        self.next_token(); // consume TagEnd

        Ok(Element {
            name,
            attributes,
            children,
        })
    }

    // fn parse_text(&mut self) -> Result<Option<Node<'a>>, ParseError<'a>> {
    //     if let Some(TokenKind::Text(text)) = self.current_token().map(|t| &t.kind) {
    //         self.next_token(); // consume Text
    //         return Ok(Some(Node::Text(text)));
    //     }
    //     Ok(None)
    // }

    fn parse_node(&mut self) -> Result<Option<Node<'a>>, ParseError<'a>> {
        match self.current_token().map(|t| &t.kind) {
            Some(TokenKind::TagStart) => self.parse_element().map(|e| Some(Node::Element(e))),
            Some(TokenKind::Text(text)) => Ok(Some(Node::Text(text))),
            _ => Ok(None),
        }
    }

    pub fn parse(&mut self) -> Result<Vec<Node<'a>>, ParseError<'a>> {
        let mut nodes = Vec::new();
        while self.position < self.tokens.len() {
            if let Some(node) = self.parse_node()? {
                nodes.push(node);
            } else {
                break;
            }
        }
        Ok(nodes)
    }
}

#[cfg(test)]
mod test {
    use crate::{
        lexer::Lexer,
        parser2::{Element, Node},
    };

    use super::Parser;

    fn iter_ast(node: &Node, depth: &mut usize) {
        let spaces = (0..*depth).into_iter().fold(String::new(), |mut s, _| {
            s.push(' ');
            s
        });

        match node {
            Node::Element(Element {
                name,
                attributes,
                children,
            }) => {
                let attrs = attributes.iter().fold(String::new(), |mut s, a| {
                    s.push_str(&format!("{}: {:?}", a.name, a.value));
                    s
                });

                println!("{spaces}{name} - {attrs}",);
                *depth += 1;

                for child in children {
                    iter_ast(child, depth);
                }
            }
            Node::Text(text) => println!("{spaces}\"{text}\""),
        }
    }

    fn lex_and_parse(input: &str) -> Result<(), String> {
        let mut lexer = Lexer::new(&input);
        let tokens = lexer.lex();

        let mut parser = Parser::new(tokens);
        let parse_results = parser.parse().unwrap();

        println!("{}", parse_results.len());

        // let mut depth = 0;

        // for ast in &parse_results {
        //     iter_ast(&ast, &mut depth);
        // }

        assert!(parse_results.len() == 1);

        Ok(())
    }

    #[test]
    fn parser_small() {
        let input = std::fs::read_to_string("./small.fml").unwrap();
        assert!(lex_and_parse(&input).is_ok());
    }

    #[test]
    fn parser_large() {
        let input = std::fs::read_to_string("./large.fml").unwrap();
        assert!(lex_and_parse(&input).is_ok());
    }

    #[test]
    fn parser_huge() {
        let input = std::fs::read_to_string("./huge.fml").unwrap();
        assert!(lex_and_parse(&input).is_ok());
    }
}
