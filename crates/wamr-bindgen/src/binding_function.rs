use proc_macro2::TokenStream;
use syn::{parse2, FnArg, Ident, ImplItemFn, ItemFn, Type};

use crate::{function_trait::*, types::WAMRTypes};
use quote::{format_ident, quote, ToTokens};
use std::ops::Deref;

pub struct BindingFunction {
    identifier: Ident,
    arguments: Vec<(Ident, Type, WAMRTypes)>,
    return_value: Option<(Type, WAMRTypes)>,
    structure: Option<Ident>,
}

impl BindingFunction {
    pub fn new_function(function: &ItemFn) -> Self {
        let function = Box::new(function.clone()) as Box<dyn FunctionTrait>;

        let mut binding_function = BindingFunction {
            identifier: function.get_identifier(),
            arguments: vec![],
            return_value: None,
            structure: None,
        };

        binding_function.build_function_signature(function);

        binding_function
    }

    pub fn new_impl_function(function: &ImplItemFn, structure: &Ident) -> Self {
        let function = Box::new(function.clone()) as Box<dyn FunctionTrait>;

        let mut binding_function = BindingFunction {
            identifier: function.get_identifier(),
            arguments: vec![],
            return_value: None,
            structure: Some(structure.clone()),
        };

        binding_function.build_function_signature(function);

        binding_function
    }

    fn build_function_signature(&mut self, function: Box<dyn FunctionTrait>) {
        self.arguments = function
            .get_arguments()
            .iter()
            .map(|argument| match argument {
                FnArg::Typed(typed) => {
                    let identifier = match typed.pat.deref() {
                        syn::Pat::Ident(ident) => ident.ident.clone(),
                        _ => panic!("Unsupported argument identifier"),
                    };

                    let wamr_type = WAMRTypes::from(typed.ty.deref());

                    (identifier, typed.ty.deref().clone(), wamr_type)
                }
                FnArg::Receiver(_) => {
                    let ty = self.structure.as_ref().expect("Receiver type not found");
                    let ty = parse2::<Type>(quote! { &mut #ty }).unwrap();

                    (format_ident!("__self"), ty, WAMRTypes::Pointer)
                }
            })
            .collect();

        self.return_value = function.get_return_type().as_ref().map(|return_type| {
            let wamr_type = WAMRTypes::from(return_type);
            (return_type.clone(), wamr_type)
        });
    }

    fn arguments_type_string(&self) -> String {
        self.arguments
            .iter()
            .map(|(_, _, wamr_type)| wamr_type.to_string())
            .collect()
    }

    pub fn get_casts(&self) -> TokenStream {
        let casts = self
            .arguments
            .iter()
            .map(|(identifier, destination, wamr_type)| {
                wamr_type.get_cast(identifier.to_token_stream(), destination)
            })
            .collect::<Vec<TokenStream>>();

        quote! {
            #(#casts)*
        }
    }

    pub fn get_signature_declaration(&self) -> TokenStream {
        let signature = self
            .return_value
            .as_ref()
            .map_or(self.arguments_type_string(), |(_, r)| {
                format!("({}){}", self.arguments_type_string(), r.to_string())
            });

        let name = format_ident!(
            "__WAMR_BINDGEN_{}_SIGNATURE",
            self.identifier.to_string().to_uppercase()
        );

        quote! {
            const #name: &'static str = #signature
        }
    }

    fn get_binding_function_name(&self) -> TokenStream {
        let name = self
            .structure
            .as_ref()
            .map_or(format_ident!("__wamr_bindgen_{}", self.identifier), |s| {
                format_ident!("__wamr_bindgen_{}_{}", s, self.identifier)
            });

        quote! { #name }
    }

    fn get_function_call(&self) -> TokenStream {
        let function_name = format_ident!("{}", self.identifier);
        let call_arguments = self
            .arguments
            .iter()
            .map(|(identifier, _, _)| quote! { #identifier });

        let function_call = quote! {
            #function_name(#(#call_arguments),*)
        };

        let function_call = if self.structure.is_some() {
            let structure = self.structure.as_ref().unwrap();
            quote! {
                #structure::#function_call
            }
        } else {
            function_call
        };

        self.return_value
            .as_ref()
            .map_or(function_call.clone(), |(_, r)| {
                let cast: Type = r.into();
                quote! {
                    #function_call as #cast
                }
            })
    }

    pub fn get_binding_function(&self) -> TokenStream {
        let binding_function_name = self.get_binding_function_name();
        let casts = self.get_casts();

        let binding_function_arguments = self.arguments.iter().map(|(identifier, _, wamr_type)| {
            let binding_type: Type = wamr_type.get_binding_type();
            quote! { #identifier: #binding_type }
        });

        let return_value = self.return_value.as_ref().map_or(quote! {}, |(_, r)| {
            let binding_type: Type = r.get_binding_type();
            quote! { #binding_type }
        });

        let function_call = self.get_function_call();

        quote! {
            #[no_mangle]
            pub unsafe extern "C" fn #binding_function_name(__wamr_environment : wamr_sys::wasm_exec_env_t, #(#binding_function_arguments),*) -> #return_value {
                let __wamr_environment = wamr_rust_sdk::execution_environment::ExecutionEnvironment::from(__wamr_environment);
                let __wamr_instance = __wamr_environment.get_instance();

                #casts

                #function_call
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use syn::parse_str;

    use syn::ItemFn;

    use super::*;

    fn get_function_a() -> ItemFn {
        parse_str::<ItemFn>(
            r#"
        fn test_function(a: i32, b: f32, c: String, d: &str, e: i64, f: u64, g: i8, h: u8, i: i16, j: u16, k: f64, l: &mut Test) {
            println!("Hello World");
        }
        "#,).unwrap()
    }

    fn get_function_b() -> ItemFn {
        parse_str::<ItemFn>(
            r#"
        fn test_function(a: i32, b: f32, c: String, d: &str, e: i64, f: u64, g: i8, h: u8, i: i16, j: u16, k: f64, l: &mut Test) -> f64 {
            println!("Hello World");
            0.0
        }
        "#,).unwrap()
    }

    #[test]
    fn test_get_wamr_function_signature() {
        let binding_function = BindingFunction::new_function(&get_function_a());
        let signature = binding_function.get_signature_declaration();
        assert_eq!(
            signature.to_string(),
            "const __WAMR_BINDGEN_TEST_FUNCTION_SIGNATURE : & 'static str = \"if$$IIiiiiF*\""
        );

        let binding_function = BindingFunction::new_function(&get_function_b());
        let signature = binding_function.get_signature_declaration();
        assert_eq!(
            signature.to_string(),
            "const __WAMR_BINDGEN_TEST_FUNCTION_SIGNATURE : & 'static str = \"(if$$IIiiiiF*)F\""
        );
    }
}
