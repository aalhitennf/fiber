mod style;

use proc_macro2::Span;
use style::{parse_enum_variant, ParsedVariants};

use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, Data, DeriveInput, Ident, ItemFn, ReturnType};

#[allow(clippy::missing_panics_doc)]
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
        .filter_map(parse_enum_variant)
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
pub fn main(_attr: TokenStream, item: TokenStream) -> TokenStream {
    let input = parse_macro_input!(item as ItemFn);

    let fn_name = &input.sig.ident; // Function name

    if fn_name.to_string() != "main" {
        panic!("fiber::main must be derived on main function!");
    }

    // let fn_vis = &input.vis; // Function visibility
    // let fn_inputs = &input.sig.inputs; // Function inputs

    let fn_output = &input.sig.output; // Function output
    let scope = input.block;

    quote! {
        fn #fn_name() #fn_output {

            #scope
        }
    }
    .into()
}

#[proc_macro_attribute]
pub fn func(_attr: TokenStream, item: TokenStream) -> TokenStream {
    let input = parse_macro_input!(item as ItemFn);

    let fn_name = &input.sig.ident; // Function name

    if fn_name.to_string() == "main" {
        panic!("fiber::func cannot be derived on main function!");
    }

    // let fn_vis = &input.vis; // Function visibility
    let fn_inputs = &input.sig.inputs; // Function inputs

    // if !fn_inputs.is_empty() {
    //     panic!("This function cannot have arguments. Use use_context to access state.");
    // }

    let fn_output = &input.sig.output; // Function output

    if !matches!(fn_output, ReturnType::Default) {
        panic!("This function cannot return a value. Use use_context to access state.");
    }

    let scope = input.block;

    let fn_name_string = fn_name.to_string();
    let fn_name_wrapper_string = format!("_fibr_{}", fn_name_string);
    let fn_name_wrapper = Ident::new(&fn_name_wrapper_string, Span::mixed_site());

    println!("fn name: {:?}", fn_name_string);

    quote! {
        fn #fn_name_wrapper(#fn_inputs) {
            #scope
        }

        fn #fn_name() -> (String, fn(std::sync::Arc<parking_lot::RwLock<State>>)) {
            (#fn_name_wrapper_string.to_string(), #fn_name_wrapper)
        }


    }
    .into()
}
