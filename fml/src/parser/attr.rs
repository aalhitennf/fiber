#[derive(Debug)]
pub struct Attribute<'a> {
    pub name: &'a str,
    pub value: AttributeValue<'a>,
}

pub enum AttributeKind<'a> {
    Class(&'a str),
}

#[derive(Debug)]
pub enum AttributeValue<'a> {
    String { value: &'a str, line: usize, col: usize },
    Integer { value: i64, line: usize, col: usize },
    Float { value: f64, line: usize, col: usize },
}

impl<'a> AttributeValue<'a> {
    #[inline]
    pub fn new(input: &'a str, line: usize, col: usize) -> Result<AttributeValue, String> {
        if let Ok(value) = input.parse::<i64>() {
            return Ok(AttributeValue::Integer { value, line, col });
        }

        if let Ok(value) = input.parse::<f64>() {
            return Ok(AttributeValue::Float { value, line, col });
        }

        if input.contains('\n') {
            return Err(format!(
                "Line breaks are not allowed in attribute values. Line {line}, col {col}"
            ));
        }

        Ok(AttributeValue::String {
            value: input.trim_matches(['"', ' ']),
            line,
            col,
        })
    }
}
