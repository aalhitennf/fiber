#![allow(clippy::missing_panics_doc)]

mod style;
mod task;

use style::{parse_enum_variant, ParsedVariants};

use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, Data, DeriveInput, ItemFn};

#[proc_macro_derive(StyleParser, attributes(key, parser, prop))]
pub fn derive_style_parser(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let name = input.ident.clone();

    let generics = input.generics;
    let (impl_generics, ty_generics, where_clause) = generics.split_for_impl();

    let Data::Enum(e) = input.data else {
        panic!("StyleParser can be only derived to enum")
    };

    let ParsedVariants {
        idents,
        names,
        parsers,
        props,
    } = e
        .variants
        .iter()
        .map(parse_enum_variant)
        .fold(ParsedVariants::default(), |mut v, p| {
            v.add(p);
            v
        });

    quote! {
        impl #impl_generics TryFrom<StyleProperty> for #name #ty_generics #where_clause {
            type Error = crate::theme::parser::StyleError;
            fn try_from(value: crate::theme::parser::StyleProperty) -> Result<Self, Self::Error> {
                match value.key.as_str() {
                    #( #names => Ok(#name::#idents(#parsers(&value.value)?)), )*
                    val @ _ => Err(crate::theme::parser::StyleError::new("Unknown style key", val)),
                }
            }
        }

        impl #impl_generics #name #ty_generics #where_clause {
            pub fn apply_transition(key: &str, t: floem::style::Transition, s: floem::style::Style) -> floem::style::Style {
                match key {
                    #( #names => s.transition(#props, t), )*
                    val @ _ => {
                        eprintln!("Invalid transition key {val}");
                        s
                    },
                }
            }
        }
    }
    .into()
}

#[proc_macro_attribute]
pub fn task(attr: TokenStream, item: TokenStream) -> TokenStream {
    let input = parse_macro_input!(item as ItemFn);
    let mut attrs = attr
        .into_iter()
        .map(|ts| ts.to_string())
        .collect::<Vec<_>>();

    let fn_pointer_path = if attrs.contains(&"debug".to_string()) {
        quote! { crate::state::FnPointer }
    } else {
        quote! { fiber::state::FnPointer }
    };

    attrs.retain(|v| v != "debug");

    if input.sig.asyncness.is_some() {
        task::build_async_task(&input, &attrs, &fn_pointer_path)
    } else {
        task::build_sync_task(&input, &fn_pointer_path)
    }
}
