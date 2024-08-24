use std::borrow::Cow;
use std::fmt::Display;
use std::sync::atomic::{AtomicU64, Ordering};

use crate::parser::Attribute;
use crate::AttributeValue;

use super::attr::VariableRef;

#[derive(Debug, Clone)]
pub enum Node<'a> {
    Element(Element<'a>),
    // Text(&'a str),
    Text(TextElement<'a>),
}

#[derive(Debug, Clone)]
pub struct TextElement<'a> {
    pub content: &'a str,
    pub variable_refs: Vec<VariableRef<'a>>,
}

#[derive(Debug, Clone, Copy)]
pub struct ElementId(u64);

pub(crate) static ELEMENT_ID: AtomicU64 = AtomicU64::new(0);

impl ElementId {
    pub fn next() -> Self {
        ElementId(ELEMENT_ID.fetch_add(1, Ordering::Relaxed))
    }

    pub fn reset() {
        ELEMENT_ID.store(0, Ordering::Relaxed);
    }
}

impl Display for ElementId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

#[derive(Debug, Clone)]
pub struct Element<'a> {
    pub id: ElementId,
    pub kind: ElementKind<'a>,
    pub attributes: Vec<Attribute<'a>>,
    pub children: Vec<Node<'a>>,
}

#[derive(Debug, Clone)]
pub enum ElementKind<'a> {
    Root,
    Box,
    VStack,
    HStack,
    Clip,
    List,
    Label,
    Button,
    Input,
    Image,
    Empty,
    Custom(Cow<'a, str>),
}

impl<'a> Element<'a> {
    #[must_use]
    pub fn new(
        name: &'a str,
        attributes: Vec<Attribute<'a>>,
        children: Vec<Node<'a>>,
    ) -> Element<'a> {
        let kind = match name.as_bytes() {
            b"root" => ElementKind::Root,
            b"box" => ElementKind::Box,
            b"vstack" => ElementKind::VStack,
            b"hstack" => ElementKind::HStack,
            b"clip" => ElementKind::Clip,
            b"list" => ElementKind::List,
            b"label" => ElementKind::Label,
            b"button" => ElementKind::Button,
            b"input" => ElementKind::Input,
            b"image" => ElementKind::Image,
            b"" => ElementKind::Empty,
            _ => ElementKind::Custom(Cow::Borrowed(name)),
        };

        Element {
            id: ElementId::next(),
            kind,
            attributes,
            children,
        }
    }

    #[must_use]
    pub fn get_attr(&self, name: &str) -> Option<AttributeValue<'_>> {
        self.attributes
            .iter()
            .find(|a| a.name == name)
            .map(|a| a.value)
    }
}
