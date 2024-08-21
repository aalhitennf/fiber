use std::borrow::Cow;
use std::fmt::Display;

use crate::TokenKind;

#[derive(Debug, Clone)]
pub struct Attribute<'a> {
    pub name: Cow<'a, str>,
    pub value: AttributeValue<'a>,
}

#[derive(Debug, Clone, Copy)]
pub struct VariableName<'a> {
    pub name: &'a str,
    pub kind: VariableType,
}

impl Display for VariableName<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.name)
    }
}

impl<'a> VariableName<'a> {
    fn kind(&self) -> VariableType {
        self.kind
    }
}

#[derive(Debug, Clone, Copy)]
pub enum VariableType {
    String,
    Integer,
    Float,
    Unknown,
}

impl<'a> From<&'a str> for VariableName<'a> {
    fn from(value: &'a str) -> Self {
        let Some((t, name)) = value.split_once(':') else {
            return VariableName {
                name: value,
                kind: VariableType::Unknown,
            };
        };

        VariableName {
            name,
            kind: VariableType::from(t),
        }
    }
}

impl<'a> From<&'a str> for VariableType {
    fn from(value: &'a str) -> Self {
        match value {
            "str" => VariableType::String,
            "int" => VariableType::Integer,
            "dec" => VariableType::Float,
            _ => VariableType::Unknown,
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct VariableRef<'a> {
    pub full_match: &'a str,
    pub start: usize,
    pub end: usize,
    pub kind: VariableType,
}

impl VariableRef<'_> {
    pub fn name(&self) -> &'_ str {
        &self.full_match[self.start + 1..self.end - 1]
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

impl Display for AttributeValue<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AttributeValue::String { value, .. } => write!(f, "{value}"),
            AttributeValue::Integer { value, .. } => write!(f, "{value}"),
            AttributeValue::Float { value, .. } => write!(f, "{value}"),
            AttributeValue::Variable { name, .. } => write!(f, "{name}"),
        }
    }
}

impl<'a> AttributeValue<'a> {
    /// # Errors
    /// Returns an error if the input is not a valid `AttributeValue`
    #[inline]
    pub fn new(input: &'a str, line: usize, col: usize) -> Result<AttributeValue, String> {
        if input.contains(':') {
            let name = VariableName::from(input.trim_end_matches(['{', '}']));
            return Ok(AttributeValue::Variable { name, line, col });
        }

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

    /// # Panics
    ///  Panics if the token is not valid `AttributeValue`
    #[inline]
    #[must_use]
    pub fn from_token(token: &TokenKind<'a>, line: usize, col: usize) -> Self {
        match token {
            TokenKind::Variable(value) => AttributeValue::Variable {
                name: VariableName::from(*value),
                line,
                col,
            },
            TokenKind::AttributeValue(value) => AttributeValue::new(value, line, col).unwrap(),
            _ => panic!("Invalid token {token:?}, expecting Variable or AttributeValue"),
            // _ => AttributeValue::String { value: AttributeValue::, line, col }
        }
    }
}
