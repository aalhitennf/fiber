use fml::{
    lexer::Lexer,
    parser::{Element, Node, Parser},
};

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
