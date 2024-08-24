use crate::lexer::TokenKind;

pub enum ParseErrorKind {
    ExpectedToken(String),
    MismatchingClosingTag(String),
}

pub struct ParseError {
    kind: ParseErrorKind,
    line: usize,
    col: usize,
}

impl ParseError {
    #[inline]
    pub fn expected_token(
        expected: &TokenKind,
        found: &TokenKind,
        line: usize,
        col: usize,
    ) -> Self {
        ParseError {
            kind: ParseErrorKind::ExpectedToken(format!(
                "Expected token `{expected:?}`, found `{found:?}` at {line}:{col}"
            )),
            line,
            col,
        }
    }

    #[inline]
    pub fn mismatching_closing_tag(expected: &str, found: &str, line: usize, col: usize) -> Self {
        ParseError {
            kind: ParseErrorKind::MismatchingClosingTag(format!(
                "Mismatching closing tag. Expected `{expected}`, found `{found}` at {line}:{col}"
            )),
            line,
            col,
        }
    }
}
