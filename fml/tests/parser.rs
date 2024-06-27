use fml::{
    lexer::Lexer,
    parser::{Element, Node, Parser},
};

fn iter_ast(node: &Node, buf: &mut String, depth: &mut usize) {
    let spaces = (0..*depth).into_iter().fold(String::new(), |mut s, _| {
        s.push_str("    ");
        s
    });

    match node {
        Node::Element(Element {
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

fn lex_and_parse(input: &str, name: &str) -> Result<(), String> {
    let mut lexer = Lexer::new(&input);
    let tokens = lexer.lex();

    let mut parser = Parser::new(tokens);
    let ast_vec = parser.parse().unwrap();

    let mut buf = String::with_capacity(1000 * 1000);

    let mut depth = 0;

    for ast in &ast_vec {
        iter_ast(ast, &mut buf, &mut depth);
    }

    std::fs::write(format!("./tests/out/{name}.ast"), buf).unwrap();

    assert!(ast_vec.len() == 1);

    Ok(())
}

#[test]
fn parse_small() {
    let input = std::fs::read_to_string("./tests/data/small.fml").unwrap();
    assert!(lex_and_parse(&input, "small").is_ok());
}

#[test]
fn parse_large() {
    let input = std::fs::read_to_string("./tests/data/large.fml").unwrap();
    assert!(lex_and_parse(&input, "large").is_ok());
}

#[test]
fn parse_huge() {
    let input = std::fs::read_to_string("./tests/data/huge.fml").unwrap();
    assert!(lex_and_parse(&input, "huge").is_ok());
}
