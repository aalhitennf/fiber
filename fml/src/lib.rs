#![allow(dead_code)]
#![allow(clippy::module_name_repetitions)]

mod lexer;
mod parser;

pub use lexer::{Lexer, Token, TokenKind};
pub use parser::{Attribute, AttributeValue, Element, ElementKind, Node, Parser};

pub fn parse(source: &str) -> Result<Node, String> {
    let mut lexer = Lexer::new(source);
    let tokens = lexer.lex();

    let mut parser = Parser::new(tokens);
    let nodes = parser.parse()?;

    if nodes.len() > 1 {
        eprintln!("There can be only one top-level tag! Using first.");
    }

    let first = nodes.into_iter().next().ok_or("No root tag found!")?;

    Ok(first)
}
