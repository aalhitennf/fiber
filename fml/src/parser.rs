use crate::lexer::{Lexer, Token, TokenKind};

#[derive(Debug)]
struct Attribute {
    name: String,
    value: String,
}

#[derive(Debug, Clone)]
pub enum AttributeValue {
    String(String),
    Float(f64),
    Int(i64),
}

#[derive(Debug)]
struct Element {
    name: String,
    attributes: Vec<Attribute>,
    children: Vec<Node>,
}

#[derive(Debug)]
enum Node {
    Element(Element),
    Text(String),
}

// struct Parser {
//     tokens: Vec<Token>,
//     position: usize,
// }

// impl Parser {
//     fn new(tokens: Vec<Token>) -> Self {
//         Parser {
//             tokens,
//             position: 0,
//         }
//     }

//     fn current_token(&self) -> Option<&Token> {
//         self.tokens.get(self.position)
//     }

//     fn advance(&mut self) {
//         self.position += 1;
//     }

//     fn parse(&mut self) -> Result<Node, String> {
//         self.parse_element()
//     }

//     fn parse_element(&mut self) -> Result<Node, String> {
//         // Expecting a tag start
//         if let Some(Token::TagStart) = self.current_token() {
//             self.advance();
//         } else {
//             return Err("Expected TagStart".to_string());
//         }

//         // Parse the tag name
//         let name = if let Some(Token::TagName(name)) = self.current_token() {
//             let name = name.clone();
//             self.advance();
//             name
//         } else {
//             return Err("Expected TagName".to_string());
//         };

//         // Parse attributes
//         let mut attributes = Vec::new();
//         while let Some(Token::AttributeName(attr_name)) = self.current_token() {
//             let name = attr_name.clone();
//             self.advance();

//             if let Some(Token::Equal) = self.current_token() {
//                 self.advance();
//             } else {
//                 return Err("Expected Equal".to_string());
//             }

//             if let Some(Token::AttributeValue(attr_value)) = self.current_token() {
//                 let value = attr_value.clone();
//                 self.advance();
//                 attributes.push(Attribute { name, value });
//             } else {
//                 return Err("Expected AttributeValue".to_string());
//             }
//         }

//         // Check for self-closing tag
//         if let Some(Token::TagSelfClose) = self.current_token() {
//             self.advance();
//             return Ok(Node::Element(Element {
//                 name,
//                 attributes,
//                 children: Vec::new(),
//             }));
//         }

//         // Expecting a tag end
//         if let Some(Token::TagEnd) = self.current_token() {
//             self.advance();
//         } else {
//             return Err("Expected TagEnd".to_string());
//         }

//         // Parse children
//         let mut children = Vec::new();
//         while let Some(token) = self.current_token() {
//             match token {
//                 Token::TagStart => {
//                     if let Some(Token::TagClose) = self.tokens.get(self.position + 1) {
//                         break;
//                     }
//                     children.push(self.parse_element()?);
//                 }
//                 Token::Text(text) => {
//                     children.push(Node::Text(text.clone()));
//                     self.advance();
//                 }
//                 _ => break,
//             }
//         }

//         // Expecting a tag close
//         if let Some(Token::TagClose) = self.current_token() {
//             self.advance();
//         } else {
//             return Err("Expected TagClose".to_string());
//         }

//         // Expecting the same tag name
//         if let Some(Token::TagName(close_name)) = self.current_token() {
//             if close_name != &name {
//                 return Err(format!(
//                     "Mismatched closing tag: expected {}, found {}",
//                     name, close_name
//                 ));
//             }
//             self.advance();
//         } else {
//             return Err("Expected TagName".to_string());
//         }

//         // Expecting a tag end
//         if let Some(Token::TagEnd) = self.current_token() {
//             self.advance();
//         } else {
//             return Err("Expected TagEnd".to_string());
//         }

//         Ok(Node::Element(Element {
//             name,
//             attributes,
//             children,
//         }))
//     }
// }

struct Parser {
    tokens: Vec<Token>,
    position: usize,
}

impl Parser {
    fn new(tokens: Vec<Token>) -> Self {
        Parser {
            tokens,
            position: 0,
        }
    }

    fn current_token(&self) -> Option<&Token> {
        self.tokens.get(self.position)
    }

    fn advance(&mut self) {
        self.position += 1;
    }

    fn parse(&mut self) -> Result<Node, String> {
        self.parse_element()
    }

    fn parse_element(&mut self) -> Result<Node, String> {
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
        let name = if let Some(Token {
            kind: TokenKind::TagName(name),
            ..
        }) = self.current_token()
        {
            let name = name.clone();
            self.advance();
            name
        } else {
            return Err("Expected TagName".to_string());
        };

        // Parse attributes
        let mut attributes = Vec::new();
        while let Some(Token {
            kind: TokenKind::AttributeName(attr_name),
            ..
        }) = self.current_token()
        {
            let name = attr_name.clone();
            self.advance();

            if let Some(Token {
                kind: TokenKind::Equal,
                ..
            }) = self.current_token()
            {
                self.advance();
            } else {
                return Err("Expected Equal (=)".to_string());
            }

            if let Some(Token {
                kind: TokenKind::AttributeValue(attr_value),
                ..
            }) = self.current_token()
            {
                let value = attr_value.clone();
                self.advance();
                attributes.push(Attribute { name, value });
            } else {
                return Err("Expected AttributeValue".to_string());
            }
        }

        // Check for self-closing tag
        if let Some(Token {
            kind: TokenKind::TagSelfClose,
            ..
        }) = self.current_token()
        {
            self.advance();
            return Ok(Node::Element(Element {
                name,
                attributes,
                children: Vec::new(),
            }));
        }

        // Expecting a tag end
        if let Some(Token {
            kind: TokenKind::TagEnd,
            ..
        }) = self.current_token()
        {
            self.advance();
        } else {
            return Err("Expected TagEnd".to_string());
        }

        // Parse children
        let mut children = Vec::new();
        while let Some(token) = self.current_token() {
            match &token.kind {
                TokenKind::TagStart => {
                    if let Some(Token {
                        kind: TokenKind::TagClose,
                        ..
                    }) = self.tokens.get(self.position + 1)
                    {
                        break;
                    }
                    children.push(self.parse_element()?);
                }
                TokenKind::Text(text) => {
                    children.push(Node::Text(text.clone()));
                    self.advance();
                }
                _ => break,
            }
        }

        // Expecting a tag close
        if let Some(Token {
            kind: TokenKind::TagClose,
            ..
        }) = self.current_token()
        {
            self.advance();
        } else {
            return Err("Expected TagClose".to_string());
        }

        // Expecting the same tag name
        if let Some(Token {
            kind: TokenKind::TagName(close_name),
            ..
        }) = self.current_token()
        {
            if close_name != &name {
                return Err(format!(
                    "Mismatched closing tag: expected {}, found {}",
                    name, close_name
                ));
            }
            self.advance();
        } else {
            return Err("Expected TagName".to_string());
        }

        // Expecting a tag end
        if let Some(Token {
            kind: TokenKind::TagEnd,
            ..
        }) = self.current_token()
        {
            self.advance();
        } else {
            return Err("Expected TagEnd".to_string());
        }

        Ok(Node::Element(Element {
            name,
            attributes,
            children,
        }))
    }
}

// fn parse_attribute_value()

#[test]
fn parser() {
    let input = std::fs::read_to_string("./test.fml").unwrap();

    let mut lexer = Lexer::new(input);
    let tokens = lexer.lex();

    // for token in &tokens {
    //     println!("{:?}", token);
    // }

    let mut parser = Parser::new(tokens);
    match parser.parse() {
        Ok(ast) => println!("{:#?}", ast),
        Err(e) => println!("Error: {}", e),
    }
}
