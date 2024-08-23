use proc_macro2::Span;
use quote::quote;
use syn::punctuated::Punctuated;
use syn::token::Comma;
use syn::{FnArg, Ident, ItemFn, PatType, ReturnType, Type};

pub(crate) fn build_async_task(
    input: &ItemFn,
    attrs: &[String],
    fn_pointer_path: &proc_macro2::TokenStream,
) -> proc_macro::TokenStream {
    let callback_name = attrs
        .first()
        .unwrap_or_else(|| panic!("You must give callback fn as attribute!"));

    let callback_fn = Ident::new(callback_name, Span::call_site());

    let fn_name = &input.sig.ident;
    assert!(fn_name != "main", "fiber::func cannot be derived on main function!");

    let ReturnType::Type(_, output_ty) = &input.sig.output else {
        panic!("fiber::async_func must have a return type!")
    };

    let fn_name_string = fn_name.to_string();
    let fn_name_wrapper_string = format!("_fibr_{fn_name_string}");
    let fn_name_wrapper = Ident::new(&fn_name_wrapper_string, Span::call_site());

    let block = &input.block;

    quote! {
        fn #fn_name_wrapper() {
            let task = async {
                #block
            };

            let task = fiber::task::AsyncTask::<#output_ty>::new(
                task,
                #callback_fn,
            );

            fiber::task::spawn(task);
        }

        fn #fn_name() -> (String, #fn_pointer_path) {
            (#fn_name_wrapper_string.to_string(), #fn_name_wrapper)
        }
    }
    .into()
}

pub(crate) fn build_sync_task(input: &ItemFn, fn_pointer_path: &proc_macro2::TokenStream) -> proc_macro::TokenStream {
    let fn_name = &input.sig.ident;

    let (names, types) = parse_inputs(&input.sig.inputs);

    let injects = quote! {
        #(let #names = floem::reactive::use_context::<#types>().expect(&format!("Context item {} not configured", stringify!(#names)));)*
    };

    assert!(
        matches!(&input.sig.output, ReturnType::Default),
        "This function cannot return a value. Use use_context to access state."
    );

    let block = &input.block;

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

fn parse_inputs(inputs: &Punctuated<FnArg, Comma>) -> (Vec<&Ident>, Vec<&Type>) {
    let mut names = Vec::with_capacity(inputs.len());
    let mut types = Vec::with_capacity(inputs.len());

    for input in inputs {
        match input {
            FnArg::Typed(PatType { pat, ty, .. }) => {
                if let syn::Pat::Ident(ident) = &**pat {
                    names.push(&ident.ident);
                    types.push(&**ty);
                } else {
                    panic!("Only named arguments are allowed in fiber::func");
                }
            }
            FnArg::Receiver(_) => panic!("Only named arguments are allowed in fiber::func"),
        }
    }

    (names, types)
}
