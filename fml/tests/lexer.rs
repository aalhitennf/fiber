use fml::Lexer;

#[inline]
fn lex_and_save(input: &str, name: &'static str) -> usize {
    let mut lexer = Lexer::new(input);
    let tokens = lexer.lex();

    let mut buf = String::with_capacity(1000 * 1000);

    for token in &tokens {
        buf.push_str(&format!("{:?}\n", token.kind));
    }

    std::fs::write(format!("./tests/out/{name}.lex"), buf).unwrap();

    tokens.len()
}

#[test]
fn lex_small() {
    let content = std::fs::read_to_string("./tests/data/small.fml").unwrap();

    let len = lex_and_save(&content, "small");
    println!("{len} tokens");

    assert!(len > 0)
}

#[test]
fn lex_large() {
    let content = std::fs::read_to_string("./tests/data/large.fml").unwrap();

    let len = lex_and_save(&content, "large");
    println!("{len} tokens");

    assert!(len == 454270)
}

#[test]
fn lex_huge() {
    let content = std::fs::read_to_string("./tests/data/huge.fml").unwrap();

    let len = lex_and_save(&content, "huge");
    println!("{len} tokens");

    assert!(len == 7100659)
}
