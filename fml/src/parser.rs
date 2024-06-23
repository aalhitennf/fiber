use nom::branch::alt;
use nom::bytes::complete::{tag, take_while};
use nom::character::complete::{alpha1, char};
use nom::combinator::{map, opt};
use nom::multi::many0;
use nom::sequence::{delimited, pair, preceded, terminated, tuple};
use nom::IResult;

#[derive(Debug, PartialEq)]
pub struct Element {
    pub name: String,
    pub attributes: Vec<(String, String)>,
    pub children: Vec<Node>,
}

#[derive(Debug, PartialEq)]
pub enum Node {
    Element(Element),
    Text(String),
}

fn identifier(input: &str) -> IResult<&str, String> {
    map(alpha1, |s: &str| s.to_string())(input)
}

fn attribute(input: &str) -> IResult<&str, (String, String)> {
    let (input, name) = identifier(input)?;
    let (input, _) = char('=')(input)?;
    let (input, value) = delimited(char('"'), take_while(|c| c != '"'), char('"'))(input)?;
    Ok((input, (name, value.to_string())))
}

fn attributes(input: &str) -> IResult<&str, Vec<(String, String)>> {
    many0(terminated(attribute, char(' ')))(input)
}

fn element(input: &str) -> IResult<&str, Element> {
    let (input, _) = char('<')(input)?;
    let (input, name) = identifier(input)?;
    let (input, attrs) = attributes(input)?;
    let (input, _) = char('>')(input)?;
    let (input, children) = many0(node)(input)?;
    let (input, _) = tag("</")(input)?;
    let (input, _) = tag(name.as_str())(input)?;
    let (input, _) = char('>')(input)?;
    Ok((
        input,
        Element {
            name,
            attributes: attrs,
            children,
        },
    ))
}

fn text(input: &str) -> IResult<&str, Node> {
    map(take_while(|c| c != '<'), |s: &str| {
        Node::Text(s.to_string())
    })(input)
}

fn node(input: &str) -> IResult<&str, Node> {
    alt((map(element, Node::Element), text))(input)
}

pub fn parse(input: &str) -> IResult<&str, Vec<Node>> {
    many0(node)(input)
}
