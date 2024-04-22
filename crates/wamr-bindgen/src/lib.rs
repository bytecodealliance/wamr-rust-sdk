extern crate proc_macro;

use std::ops::Deref;

use proc_macro2::TokenStream;
use quote::quote;
use syn::{parse2, ItemFn, ItemImpl};

mod types;

mod binding_function;
use binding_function::BindingFunction;

mod function_trait;

#[proc_macro_attribute]
pub fn impl_bindgen(
    _attributes: proc_macro::TokenStream,
    item: proc_macro::TokenStream,
) -> proc_macro::TokenStream {
    let item = TokenStream::from(item);

    let implementation: ItemImpl = parse2(item.clone()).unwrap();

    let identifier = match implementation.self_ty.deref() {
        syn::Type::Path(path) => path.path.segments.first().unwrap().ident.clone(),
        _ => panic!("Unsupported implementation type"),
    };

    // Get all functions from the implementation and convert them to BindingFunction

    let binding_functions: Vec<BindingFunction> = implementation
        .items
        .iter()
        .filter(|item| matches!(item, syn::ImplItem::Fn(_)))
        .map(|item| match item {
            syn::ImplItem::Fn(function) => {
                BindingFunction::new_impl_function(function, &identifier)
            }
            _ => panic!("Unsupported item type"),
        })
        .collect();

    // Generate the binding functions

    let binding_functions_declaration = binding_functions
        .iter()
        .map(|binding_function| binding_function.get_binding_function())
        .collect::<Vec<TokenStream>>();

    let gen = quote! {

        #item

        #(#binding_functions_declaration)*

    };

    proc_macro::TokenStream::from(gen)
}

#[proc_macro_attribute]
pub fn function_bindgen(
    _attributes: proc_macro::TokenStream,
    item: proc_macro::TokenStream,
) -> proc_macro::TokenStream {
    let item = TokenStream::from(item);

    let function = parse2::<ItemFn>(item.clone()).unwrap();

    let binding_function = BindingFunction::new_function(&function);

    let binding_function = binding_function.get_binding_function();

    let gen = quote! {

        #item


        #binding_function
    };

    proc_macro::TokenStream::from(gen)
}
