extern crate proc_macro;

use proc_macro::TokenStream;
use quote::quote;
use syn::{meta, parse_macro_input, FnArg, ItemFn, LitStr, ReturnType, Type};

#[proc_macro_attribute]
pub fn generate_host_function(args: TokenStream, item: TokenStream) -> TokenStream {
    let input = parse_macro_input!(item as ItemFn);

    let mut name_override: Option<LitStr> = None;
    let mut signature_override: Option<LitStr> = None;

    let args_parser = meta::parser(|meta| {
        if meta.path.is_ident("name") {
            name_override = Some(meta.value()?.parse()?);
        } else if meta.path.is_ident("signature") {
            signature_override = Some(meta.value()?.parse()?);
        } else {
            return Err(meta.error("unsupported generate host function property."));
        }
        Ok(())
    });

    parse_macro_input!(args with args_parser);

    let function_ident = &input.sig.ident;
    let function_vis = &input.vis;
    let function_inputs = &input.sig.inputs;
    let function_output = &input.sig.output;
    let function_block = &input.block;

    let function_name = name_override
        .map(|n| n.value())
        .unwrap_or_else(|| function_ident.to_string());

    let signature = signature_override.map(|s| s.value()).unwrap_or_else(|| {
        let mut param_signature = String::new();
        let mut buffer_flag = false;

        for arg in function_inputs.iter() {
            if let FnArg::Typed(typed) = arg {
                let sig_char = get_signature_for_type(&typed.ty)
                    .unwrap_or_else(|| panic!("Unsupported parameter type."));

                if sig_char == '~' && !buffer_flag {
                    panic!("`~` must follow `*` (buffer address).");
                }

                buffer_flag = sig_char == '*';
                param_signature.push(sig_char);
            }
        }

        let return_signature = match function_output {
            ReturnType::Default => String::new(),
            ReturnType::Type(_, ret_type) => get_signature_for_type(ret_type)
                .unwrap_or_else(|| panic!("Unsupported return type."))
                .to_string(),
        };

        format!("({}){}", param_signature, return_signature)
    });

    let c_function_ident = syn::Ident::new(&format!("{}_c", function_ident), function_ident.span());

    let expanded = quote! {
        #function_vis extern "C" fn #c_function_ident(exec_env: wamr_rust_sdk::sys::wasm_exec_env_t, #function_inputs) #function_output #function_block

        #function_vis fn #function_ident() -> wamr_rust_sdk::host_function::HostFunction {
            wamr_rust_sdk::host_function::HostFunction::new(
                #function_name,
                #c_function_ident as *mut core::ffi::c_void,
                #signature
            )
        }
    };

    TokenStream::from(expanded)
}

fn get_signature_for_type(ty: &Type) -> Option<char> {
    match ty {
        Type::Path(type_path) => {
            let type_name = type_path.path.segments.last()?.ident.to_string();
            match type_name.as_str() {
                "i32" | "u32" => Some('i'),
                "i64" | "u64" => Some('I'),
                "f32" => Some('f'),
                "f64" => Some('F'),
                "usize" => Some('i'),
                _ => None,
            }
        }
        Type::Reference(type_ref) => match &*type_ref.elem {
            Type::Path(type_path) => {
                let type_name = type_path.path.segments.last()?.ident.to_string();
                if type_name == "str" {
                    Some('$')
                } else {
                    None
                }
            }
            _ => None,
        },
        Type::Ptr(type_ptr) => {
            if let Type::Path(type_path) = &*type_ptr.elem {
                let type_name = type_path.path.segments.last()?.ident.to_string();
                if type_name == "u8" {
                    Some('*')
                } else {
                    None
                }
            } else {
                None
            }
        }
        _ => None,
    }
}
