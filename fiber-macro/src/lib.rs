#![allow(clippy::missing_panics_doc)]

mod func;
mod style;

use proc_macro2::Span;
use style::{parse_enum_variant, ParsedVariants};

use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, Data, DeriveInput, Fields, Ident, ItemFn, ReturnType};

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

    assert!(input.sig.asyncness.is_none(), "fiber::func cannot be async!");

    let fn_name = &input.sig.ident;
    assert!(fn_name != "main", "fiber::func cannot be derived on main function!");

    let (names, types) = func::parse_inputs(&input.sig.inputs);

    let injects = quote! {
        #(let #names = floem::reactive::use_context::<#types>().expect(&format!("Context item {} not configured", stringify!(#names)));)*
    };

    assert!(
        matches!(&input.sig.output, ReturnType::Default),
        "This function cannot return a value. Use use_context to access state."
    );

    let block = input.block;

    let fn_name_string = fn_name.to_string();
    let fn_name_wrapper_string = format!("_fibr_{fn_name_string}");
    let fn_name_wrapper = Ident::new(&fn_name_wrapper_string, Span::call_site());

    quote! {
        fn #fn_name_wrapper() {
            #injects

            #block
        }

        fn #fn_name() -> (String, #fn_pointer_path) {
            (#fn_name_wrapper_string.to_string(), #fn_name_wrapper)
        }
    }
    .into()
}

#[proc_macro_attribute]
pub fn async_func(attr: TokenStream, item: TokenStream) -> TokenStream {
    let input = parse_macro_input!(item as ItemFn);
    let mut attrs = attr.into_iter().map(|ts| ts.to_string()).collect::<Vec<_>>();

    let fn_pointer_path = if attrs.contains(&"debug".to_string()) {
        quote! { crate::state::FnPointer }
    } else {
        quote! { fiber::state::FnPointer }
    };

    attrs.retain(|v| v != "debug");

    let callback_name = attrs
        .first()
        .unwrap_or_else(|| panic!("You must give callback fn as attribute!"));

    let callback_fn = Ident::new(callback_name, Span::call_site());
    //
    assert!(
        input.sig.asyncness.is_some(),
        "fiber::async_func must be derived on async function!"
    );

    let fn_name = &input.sig.ident;
    assert!(fn_name != "main", "fiber::func cannot be derived on main function!");

    let ReturnType::Type(_, output_ty) = &input.sig.output else {
        panic!("fiber::async_func must have a return type!")
    };

    let fn_name_string = fn_name.to_string();
    let fn_name_wrapper_string = format!("_fibr_{fn_name_string}");
    let fn_name_wrapper = Ident::new(&fn_name_wrapper_string, Span::call_site());

    let block = input.block;

    quote! {
        fn #fn_name_wrapper() {
            let task = async {
                #block
            };

            let task = fiber::AsyncTask::<#output_ty>::new(
                task,
                #callback_fn,
            );

            fiber::run_task(task);
        }

        fn #fn_name() -> (String, #fn_pointer_path) {
            (#fn_name_wrapper_string.to_string(), #fn_name_wrapper)
        }
    }
    .into()
}

#[proc_macro_derive(Stateful)]
pub fn stateful(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);

    let struct_name = &input.ident;

    let fields = match &input.data {
        syn::Data::Struct(data) => match &data.fields {
            Fields::Named(fields) => fields,
            Fields::Unnamed(_) => panic!("Tuple structs are not supported"),
            Fields::Unit => panic!("Unit structs are not supported"),
        },
        _ => panic!("Only structs are supported"),
    };

    let (idents, types) = fields.named.iter().fold((vec![], vec![]), |mut acc, field| {
        acc.0.push(&field.ident);
        acc.1.push(&field.ty);
        acc
    });

    // let maps = fields.named.iter().map(|field| {
    //     // let key = &field.ident;
    //     let ty = &field.ty;
    //     let map_ident = ty.to_token_stream().to_string().to_lowercase();
    //     let map_ident = Ident::new(&map_ident, Span::call_site());

    //     quote! {
    //         #map_ident: std::collections::HashMap<String, floem::reactive::RwSignal<#ty>>,
    //     }
    // });

    quote! {
            // #(#idents: floem::reactive::RwSignal<#types>,)*
        pub struct MagicalState {
            #(#idents: floem::reactive::RwSignal<#types>,)*
        }

        impl fiber::state::Stateful for MagicalState {

        }

                    // #(stringify!(#idents) => Some(self.#idents.cloned()),)*
        // impl MagicalState {
        //     pub fn get_field<T>(&self, key: &str) -> Option<floem::reactive::RwSignal<T>> {
        //         match key {
        //             #(  stringify!(#idents) => Some(self.#idents), )*
        //             _ => None,
        //         }
        //     }
        // }

        impl #struct_name {
            pub fn build() -> MagicalState {
                let default = #struct_name::default();

                MagicalState {
                    #(#idents: floem::reactive::RwSignal::new(default.#idents),)*
                }
            }
        }
    }
    .into()
}
