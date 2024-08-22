use syn::punctuated::Punctuated;
use syn::token::Comma;
use syn::{FnArg, Ident, PatType, Type};

pub(crate) fn parse_inputs(inputs: &Punctuated<FnArg, Comma>) -> (Vec<&Ident>, Vec<&Type>) {
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
