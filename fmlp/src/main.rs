use std::path::PathBuf;

use fml::{Element, Lexer, Node, Parser};

fn main() {
    let mut args = std::env::args();
    args.next();

    let path: PathBuf = args.next().expect("No file argument").into();
    let input = std::fs::read_to_string(std::fs::canonicalize(&path).unwrap()).unwrap();
    let filename = path.file_name().unwrap().to_str().unwrap();

    let tokens_filename = format!("{filename}.tokens.txt");
    let ast_filename = format!("{filename}.ast.txt");

    let mut lexer = Lexer::new(&input);
    let tokens = lexer.lex();

    let mut lex_buf = String::with_capacity(input.len());

    for token in &tokens {
        lex_buf.push_str(&format!("{token:?}\n"));
    }

    let mut parser = Parser::new(tokens);
    let ast_vec = parser.parse().unwrap();

    let mut ast_buf = String::with_capacity(input.len());

    let mut depth = 0;

    for ast in &ast_vec {
        iter_ast(ast, &mut ast_buf, &mut depth);
    }

    std::fs::write(format!("./{tokens_filename}"), lex_buf).unwrap();
    std::fs::write(format!("./{ast_filename}"), ast_buf).unwrap();
}

fn iter_ast(node: &Node, buf: &mut String, depth: &mut usize) {
    let spaces = (0..*depth).fold(String::new(), |mut s, _| {
        s.push_str("    ");
        s
    });

    match node {
        Node::Element(Element {
            id: _,
            kind,
            attributes,
            children,
        }) => {
            let attrs = attributes.iter().fold(String::new(), |mut s, a| {
                s.push_str(&format!("{}: {:?} ", a.name, a.value));
                s
            });

            buf.push_str(&format!("{spaces}{kind:?}"));

            if !attrs.is_empty() {
                buf.push_str(&format!(" | {attrs}"));
            }

            buf.push('\n');

            *depth += 1;

            for child in children {
                iter_ast(child, buf, depth);
            }

            *depth -= 1;
        }
        Node::Text(text) => buf.push_str(&format!("{spaces}{text}\n")),
    }
}
