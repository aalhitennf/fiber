use std::borrow::Cow;

#[derive(Debug, Clone)]
pub struct Attribute<'a> {
    pub name: Cow<'a, str>,
    pub value: AttributeValue<'a>,
}

#[derive(Debug, Clone, Copy)]
pub struct VariableName<'a>(&'a str);

impl<'a> ToString for VariableName<'a> {
    fn to_string(&self) -> String {
        self.0.to_string()
    }
}

impl<'a> From<&'a str> for VariableName<'a> {
    fn from(value: &'a str) -> Self {
        VariableName(value)
    }
}

#[derive(Debug, Clone, Copy)]
pub enum AttributeValue<'a> {
    String {
        value: &'a str,
        line: usize,
        col: usize,
    },
    Integer {
        value: i64,
        line: usize,
        col: usize,
    },
    Float {
        value: f64,
        line: usize,
        col: usize,
    },
    Variable {
        name: VariableName<'a>,
        line: usize,
        col: usize,
    },
}

impl<'a> ToString for AttributeValue<'a> {
    fn to_string(&self) -> String {
        match self {
            AttributeValue::String { value, .. } => value.to_string(),
            AttributeValue::Integer { value, .. } => value.to_string(),
            AttributeValue::Float { value, .. } => value.to_string(),
            AttributeValue::Variable { name, .. } => name.to_string(),
        }
    }
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
