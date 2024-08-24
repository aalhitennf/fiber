use proc_macro2::Span;
use syn::{Ident, LitStr, Variant};

#[derive(Default)]
pub struct ParsedVariants<'a> {
    pub(crate) idents: Vec<&'a Ident>,
    pub(crate) names: Vec<LitStr>,
    pub(crate) parsers: Vec<proc_macro2::Ident>,
    pub(crate) props: Vec<Ident>,
}

impl<'a> ParsedVariants<'a> {
    pub fn add(&mut self, var: ParsedVariant<'a>) {
        self.idents.push(var.ident);
        self.names.push(var.name);
        self.parsers.push(var.parser);
        self.props.push(var.prop);
    }
}

pub struct ParsedVariant<'a> {
    ident: &'a Ident,
    name: LitStr,
    parser: proc_macro2::Ident,
    prop: Ident,
}

// # Panics
// Panics if any of the required attributes are missing
pub fn parse_enum_variant(v: &Variant) -> ParsedVariant<'_> {
    let ident = &v.ident;

    let Some(name_attr) = v.attrs.iter().find(|a| a.path().is_ident("key")) else {
        panic!("Missing key attribute for {ident}");
    };

    let Ok(lit) = name_attr.parse_args::<LitStr>() else {
        panic!("Key attribute must be string literal");
    };

    let Some(Ok(parser)) = v
        .attrs
        .iter()
        .find(|a| a.path().is_ident("parser"))
        .map(|a| {
            a.parse_args::<LitStr>()
                .map(|lit| Ident::new(&lit.value(), Span::call_site()))
        })
    else {
        panic!("Missing convert fn for {ident}");
    };

    let Some(prop_attr) = v.attrs.iter().find(|a| a.path().is_ident("prop")) else {
        panic!("Missing prop attribute for {ident}");
    };

    let Ok(prop) = prop_attr.parse_args::<Ident>() else {
        panic!("Prop attribute must be ident");
    };

    ParsedVariant {
        ident,
        name: lit,
        parser,
        prop,
    }
}
