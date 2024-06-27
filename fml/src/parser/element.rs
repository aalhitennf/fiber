use crate::parser::Attribute;

#[derive(Debug)]
pub enum Node<'a> {
    Element(Element<'a>),
    Text(&'a str),
}

#[derive(Debug)]
pub struct Element<'a> {
    pub kind: ElementKind<'a>,
    pub attributes: Vec<Attribute<'a>>,
    pub children: Vec<Node<'a>>,
}

#[derive(Debug)]
pub enum ElementKind<'a> {
    Root,
    Box,
    VStack,
    HStack,
    Clip,
    Label,
    Button,
    Input,
    Image,
    Empty,
    Custom(&'a str),
}

impl<'a> Element<'a> {
    #[must_use]
    pub const fn new(name: &'a str, attributes: Vec<Attribute<'a>>, children: Vec<Node<'a>>) -> Element<'a> {
        let kind = match name.as_bytes() {
            b"root" => ElementKind::Root,
            b"box" => ElementKind::Box,
            b"vstack" => ElementKind::VStack,
            b"hstack" => ElementKind::HStack,
            b"clip" => ElementKind::Clip,
            b"label" => ElementKind::Label,
            b"button" => ElementKind::Button,
            b"input" => ElementKind::Input,
            b"image" => ElementKind::Image,
            b"" => ElementKind::Empty,
            _ => ElementKind::Custom(name),
        };

        Element {
            kind,
            attributes,
            children,
        }
    }
}
