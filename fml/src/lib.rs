#![allow(dead_code)]
#![allow(clippy::module_name_repetitions)]

mod lexer;
mod parser;

pub use lexer::{Lexer, Token, TokenKind};
pub use parser::{
    Attribute, AttributeValue, Element, ElementKind, Node, Parser, TextElement, VariableName, VariableType,
};

/// # Errors
/// Returns an error if the source is not a valid FML
pub fn parse(source: &str) -> Result<Node, String> {
    let mut lexer = Lexer::new(source);
    let tokens = lexer.lex();

    let mut parser = Parser::new(tokens);
    let nodes = parser.parse()?;

    if nodes.len() > 1 {
        eprintln!("There can be only one top-level tag! Using first.");
    }

    if nodes.is_empty() {
        eprintln!("Parser returned no nodes");
    }

    let first = nodes.into_iter().next().ok_or("No root tag found!")?;

    Ok(first)
}
