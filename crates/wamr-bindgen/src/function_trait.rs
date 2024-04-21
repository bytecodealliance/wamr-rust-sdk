use std::ops::Deref;
use syn::{FnArg, Ident, ImplItemFn, ItemFn, Type};

pub trait FunctionTrait {
    fn get_arguments(&self) -> Vec<FnArg>;
    fn get_return_type(&self) -> Option<Type>;
    fn get_identifier(&self) -> Ident;
}

impl FunctionTrait for ItemFn {
    fn get_arguments(&self) -> Vec<FnArg> {
        self.sig.inputs.iter().cloned().collect()
    }

    fn get_identifier(&self) -> Ident {
        self.sig.ident.clone()
    }

    fn get_return_type(&self) -> Option<Type> {
        match &self.sig.output {
            syn::ReturnType::Type(_, ty) => Some(ty.deref().clone()),
            syn::ReturnType::Default => None,
        }
    }
}

impl FunctionTrait for ImplItemFn {
    fn get_arguments(&self) -> Vec<FnArg> {
        self.sig.inputs.iter().cloned().collect()
    }

    fn get_return_type(&self) -> Option<Type> {
        match &self.sig.output {
            syn::ReturnType::Type(_, ty) => Some(ty.deref().clone()),
            syn::ReturnType::Default => None,
        }
    }

    fn get_identifier(&self) -> Ident {
        self.sig.ident.clone()
    }
}
