use std::fmt::Display;

use crate::lexer::{Token, TokenKind};

#[derive(Debug)]
pub struct Attribute<'a> {
    name: &'a str,
    value: &'a str,
}

// #[derive(Debug)]
// pub enum AttributeValue {
//     String(String),
//     Float(f64),
//     Int(i64),
// }

#[derive(Debug)]
pub struct Element<'a> {
    name: &'a str,
    attributes: Vec<Attribute<'a>>,
    children: Vec<Node<'a>>,
}

#[derive(Debug)]
pub enum Node<'a> {
    Element(Element<'a>),
    Text(&'a str),
    Error(String),
}

#[derive(Debug)]
pub enum ParseError<'a> {
    ExpectedToken {
        expected: TokenKind<'a>,
        found: &'a Token<'a>,
    },
    MismatchedClosingTag {
        expected: &'a str,
        found: &'a str,
    },
}

impl<'a> ParseError<'a> {
    pub fn expected(expected: TokenKind<'a>, found: &'a Token<'a>) -> Self {
        ParseError::ExpectedToken { expected, found }
    }

    pub fn mismatched_closing_tag(expected: &'a str, found: &'a str) -> Self {
        ParseError::MismatchedClosingTag { expected, found }
    }
}

impl<'a> From<ParseError<'a>> for Node<'a> {
    fn from(value: ParseError<'a>) -> Self {
        Node::Error(value.to_string())
    }
}

impl<'a> Display for ParseError<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ParseError::ExpectedToken { expected, found } => write!(
                f,
                "Expected {} at position {}:{}, found {}",
                expected, found.start, found.end, found.kind
            ),

            ParseError::MismatchedClosingTag { expected, found } => {
                write!(f, "Mismatching closing tag. Expected {expected}, found {found}.")
            }
        }
    }
}

pub struct Parser<'a> {
    tokens: Vec<Token<'a>>,
    position: usize,
}

impl<'a> Parser<'a> {
    fn new(tokens: Vec<Token<'a>>) -> Self {
        Parser { tokens, position: 0 }
    }

    fn current_token(&self) -> Option<Token> {
        self.tokens.get(self.position).copied()
    }

    fn advance(&mut self) {
        self.position += 1;
    }

    fn parse(&mut self) -> Vec<Node<'a>> {
        let mut nodes = Vec::with_capacity(self.tokens.len());

        while let Ok(node) = self.parse_element() {
            nodes.push(node);
        }

        nodes
    }

    #[allow(clippy::too_many_lines)]
    fn parse_element(&mut self) -> Result<Node<'a>, String> {
        // Expecting a tag start
        if let Some(Token {
            kind: TokenKind::TagStart,
            ..
        }) = self.current_token()
        {
            self.advance();
        } else {
            return Err("Expected TagStart".to_string());
        }

        // Parse the tag name
        let name = if let Some(token) = self.current_token() {
            if let TokenKind::TagName(name) = token.kind {
                name
            } else {
                return Err("Expected TagName".to_string());
            }
        } else {
            return Err("Expected TagName".to_string());
        };

        self.advance();

        // Parse attributes
        let mut attributes = Vec::new();
        loop {
            if let Some(token) = self.current_token() {
                match token.kind {
                    TokenKind::AttributeName(attr_name) => {
                        self.advance();

                        // Expecting an equal sign
                        if let Some(token) = self.current_token() {
                            if !matches!(token.kind, TokenKind::EqualSign) {
                                return Err("Expected Equal (=)".to_string());
                            }
                        } else {
                            return Err("Expected Equal (=)".to_string());
                        }
                        self.advance();

                        // Expecting an attribute value
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
            } else {
                break;
            }
        }

        // Check for self-closing tag
        if let Some(token) = self.current_token() {
            if matches!(token.kind, TokenKind::TagSelfClose) {
                self.advance();

                return Ok(Node::Element(Element {
                    name,
                    attributes,
                    children: Vec::new(),
                }));
            }
        }

        // Expecting a tag end
        if let Some(token) = self.current_token() {
            if !matches!(token.kind, TokenKind::TagEnd) {
                return Err("Expected TagEnd".to_string());
            }
        } else {
            return Err("Expected TagEnd".to_string());
        }
        self.advance();

        // Parse children
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
                    children.push(self.parse_element()?);
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

        // Expecting a tag close
        if let Some(token) = self.current_token() {
            if !matches!(token.kind, TokenKind::TagClose) {
                return Err("Expected TagClose".to_string());
            }
        } else {
            return Err("Expected TagClose".to_string());
        }
        self.advance();

        // Expecting the same tag name
        if let Some(token) = self.current_token() {
            if let TokenKind::TagName(close_name) = token.kind {
                if close_name != name {
                    return Err(format!(
                        "Mismatched closing tag: expected {}, found {}",
                        name, close_name
                    ));
                }
            } else {
                return Err("Expected TagName".to_string());
            }
        } else {
            return Err("Expected TagName".to_string());
        }
        self.advance();

        // Expecting a tag end
        if let Some(token) = self.current_token() {
            if !matches!(token.kind, TokenKind::TagEnd) {
                return Err("Expected TagEnd".to_string());
            }
        } else {
            return Err("Expected TagEnd".to_string());
        }
        self.advance();

        Ok(Node::Element(Element {
            name,
            attributes,
            children,
        }))
    }
}

// fn parse_attribute_value(value: &str) -> AttributeValue {
//     if value.contains('.') {
//         if let Ok(f) = value.parse::<f64>() {
//             tokens.push(Token {
//                 kind: TokenKind::AttributeValue(AttributeValue::Float(f),
//                 start: start_pos,
//                 end: self.position,
//             });
//         } else {
//             tokens.push(Token {
//                 kind: TokenKind::Error(format!(
//                     "{value} cannot be parser as f64"
//                 )),
//                 start: start_pos,
//                 end: self.position,
//             });
//         }
//     } else {
//         if let Ok(i) = value.parse::<i64>() {
//             tokens.push(Token::AttributeValue(AttributeValue::Int(i)));
//         } else {
//             tokens
//                 .push(Token::Error(format!("{value} cannot be parser as i64")));
//         }
//     };
// }

#[cfg(test)]
mod test {
    use crate::{
        lexer::Lexer,
        parser::{Element, Node},
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
                    s.push_str(&format!("{}: {}", a.name, a.value));
                    s
                });

                println!("{spaces}{name} - {attrs}",);
                *depth += 1;

                for child in children {
                    iter_ast(child, depth);
                }
            }
            Node::Text(text) => println!("{spaces}\"{text}\""),

            Node::Error(err) => eprintln!("{err}"),
        }
    }

    fn lex_and_parse(input: &str) -> Result<(), String> {
        let mut lexer = Lexer::new(&input);
        let tokens = lexer.lex();

        let mut parser = Parser::new(tokens);
        let parse_results = parser.parse();

        println!("{}", parse_results.len());

        // let mut depth = 0;

        // for ast in &parse_results {
        //     iter_ast(&ast, &mut depth);
        // }

        // assert!(parse_results.len() == 1);

        Ok(())
    }

    #[test]
    fn parser_simple() {
        let input = std::fs::read_to_string("./simple.fml").unwrap();
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
