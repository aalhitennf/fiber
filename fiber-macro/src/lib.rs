#![allow(clippy::missing_panics_doc)]

mod style;

use proc_macro2::Span;
use style::{parse_enum_variant, ParsedVariants};

use proc_macro::TokenStream;
use quote::quote;
use syn::punctuated::Punctuated;
use syn::token::Comma;
use syn::{parse_macro_input, Data, DeriveInput, FnArg, Ident, ItemFn, PatType, ReturnType, Type};

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
pub fn func(attr: TokenStream, item: TokenStream) -> TokenStream {
    let input = parse_macro_input!(item as ItemFn);
    let attrs = attr.into_iter().map(|ts| ts.to_string()).collect::<Vec<_>>();

    let fn_pointer_path = if attrs.contains(&"debug".to_string()) {
        quote! { crate::state::FnPointer }
    } else {
        quote! { fiber::state::FnPointer }
    };

    assert!(input.sig.asyncness.is_none(), "fiber::func cannot be async! (yet)");

    let fn_name = &input.sig.ident;
    assert!(fn_name != "main", "fiber::func cannot be derived on main function!");

    let (names, types) = parse_inputs(&input.sig.inputs);

    let injects = quote! {
        #(let #names = floem::reactive::use_context::<#types>().unwrap();)*
    };

    assert!(
        matches!(&input.sig.output, ReturnType::Default),
        "This function cannot return a value. Use use_context to access state."
    );

    let scope = input.block;

    let fn_name_string = fn_name.to_string();
    let fn_name_wrapper_string = format!("_fibr_{fn_name_string}");
    let fn_name_wrapper = Ident::new(&fn_name_wrapper_string, Span::call_site());

    quote! {
        fn #fn_name_wrapper() {
            #injects

            #scope
        }

        fn #fn_name() -> (String, #fn_pointer_path) {
            (#fn_name_wrapper_string.to_string(), #fn_name_wrapper)
        }


    }
    .into()
}

fn parse_inputs(inputs: &Punctuated<FnArg, Comma>) -> (Vec<&Ident>, Vec<&Box<Type>>) {
    let mut names = Vec::with_capacity(inputs.len());
    let mut types = Vec::with_capacity(inputs.len());

    for input in inputs {
        match input {
            FnArg::Typed(PatType { pat, ty, .. }) => {
                if let syn::Pat::Ident(ident) = &**pat {
                    names.push(&ident.ident);
                    types.push(ty);
                } else {
                    panic!("Only named arguments are allowed in fiber::func");
                }
            }
            _ => panic!("Only named arguments are allowed in fiber::func"),
        }
    }

    (names, types)
}
