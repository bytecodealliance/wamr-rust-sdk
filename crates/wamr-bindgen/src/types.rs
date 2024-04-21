use std::ops::Deref;

use proc_macro2::TokenStream;
use quote::{quote, ToTokens};
use syn::{parse_str, Type};

#[derive(Debug, PartialEq, Eq)]
pub enum WAMRTypes {
    ShortIntegers,
    LongIntegers,
    ShortFloats,
    LongFloats,
    String,
    Pointer,
    NativeReference,
}

impl WAMRTypes {
    pub fn get_cast(&self, identifier: TokenStream, destination: &Type) -> TokenStream {
        match self {
            WAMRTypes::ShortFloats | WAMRTypes::LongFloats => quote! {},
            WAMRTypes::ShortIntegers | WAMRTypes::LongIntegers => {
                quote! {
                    let #identifier = #identifier as #destination;
                }
            }
            WAMRTypes::String => {
                let string_cast = quote! {
                    let #identifier = unsafe {
                        std::ffi::CStr::from_ptr(#identifier as *const i8)
                    }.to_str().unwrap()
                };

                let string_cast = match destination
                    .to_token_stream()
                    .to_string()
                    .replace(" ", "")
                    .as_str()
                {
                    "String" => quote! {
                        #string_cast.to_string();
                    },
                    "&str" => quote! {
                        #string_cast;
                    },
                    _ => panic!(
                        "Invalid destination type: {}",
                        destination.to_token_stream()
                    ),
                };

                quote! {
                    assert!( unsafe { wamr_sys::wasm_runtime_validate_app_str_addr(__wamr_instance, #identifier) } , "Invalid pointer");

                    let #identifier: *const char
                     = unsafe {
                        std::mem::transmute(wamr_sys::wasm_runtime_addr_app_to_native(__wamr_instance, #identifier))
                    };
                    #string_cast
                }
            }
            WAMRTypes::Pointer => {
                let destination_element = match destination {
                    Type::Reference(reference) => reference.elem.deref(),
                    _ => panic!(
                        "Invalid destination type: {}",
                        destination.to_token_stream()
                    ),
                };

                quote! {
                    assert!( unsafe { wamr_sys::wasm_runtime_validate_app_addr(__wamr_instance, #identifier, std::mem::size_of::<#destination_element>() as u32) } , "Invalid pointer");

                    let #identifier: #destination = unsafe {
                        std::mem::transmute(wamr_sys::wasm_runtime_addr_app_to_native(__wamr_instance, #identifier))
                    };

                }
            }
            WAMRTypes::NativeReference => {
                quote! {
                    let #identifier: #destination = unsafe { std::mem::transmute(#identifier) };
                }
            }
        }
    }

    pub fn get_binding_type(&self) -> Type {
        match self {
            WAMRTypes::ShortIntegers => parse_str("i32").unwrap(),
            WAMRTypes::LongIntegers => parse_str("i64").unwrap(),
            WAMRTypes::ShortFloats => parse_str("f32").unwrap(),
            WAMRTypes::LongFloats => parse_str("f64").unwrap(),
            WAMRTypes::String => parse_str("u32").unwrap(),
            WAMRTypes::Pointer => parse_str("u32").unwrap(),
            WAMRTypes::NativeReference => parse_str("usize").unwrap(),
        }
    }
}

impl From<&WAMRTypes> for Type {
    fn from(ty: &WAMRTypes) -> Self {
        match ty {
            WAMRTypes::ShortIntegers => parse_str("i32").unwrap(),
            WAMRTypes::LongIntegers => parse_str("i64").unwrap(),
            WAMRTypes::ShortFloats => parse_str("f32").unwrap(),
            WAMRTypes::LongFloats => parse_str("f64").unwrap(),
            WAMRTypes::String => parse_str("String").unwrap(),
            WAMRTypes::Pointer => parse_str("u32").unwrap(),
            WAMRTypes::NativeReference => parse_str("usize").unwrap(),
        }
    }
}

impl From<&Type> for WAMRTypes {
    fn from(ty: &Type) -> Self {
        match ty {
            Type::Path(ref path) => {
                let seg = &path.path.segments[0];
                match seg.ident.to_string().as_str() {
                    "u8" | "i8" | "u16" | "i16" | "u32" | "i32" => WAMRTypes::ShortIntegers,
                    "usize" | "u64" | "i64" => WAMRTypes::LongIntegers,
                    "f32" => WAMRTypes::ShortFloats,
                    "f64" => WAMRTypes::LongFloats,
                    "String" => WAMRTypes::String,
                    _ => panic!("Invalid type: {}", seg.ident),
                }
            }
            Type::Reference(ref reference) => {
                if reference.elem.deref().to_token_stream().to_string() == "str"
                    && reference.mutability.is_none()
                {
                    WAMRTypes::String
                } else {
                    WAMRTypes::Pointer
                }
            }
            _ => panic!("Invalid type : not a path"),
        }
    }
}

impl ToString for WAMRTypes {
    fn to_string(&self) -> String {
        match self {
            WAMRTypes::ShortIntegers => "i",
            WAMRTypes::LongIntegers => "I",
            WAMRTypes::ShortFloats => "f",
            WAMRTypes::LongFloats => "F",
            WAMRTypes::String => "$",
            WAMRTypes::Pointer => "*",
            WAMRTypes::NativeReference => "r",
        }
        .to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_from_type() {
        let ty: Type = parse_str("i32").unwrap();
        assert_eq!(WAMRTypes::from(&ty), WAMRTypes::ShortIntegers);

        let ty: Type = parse_str("i64").unwrap();
        assert_eq!(WAMRTypes::from(&ty), WAMRTypes::LongIntegers);

        let ty: Type = parse_str("f32").unwrap();
        assert_eq!(WAMRTypes::from(&ty), WAMRTypes::ShortFloats);

        let ty: Type = parse_str("f64").unwrap();
        assert_eq!(WAMRTypes::from(&ty), WAMRTypes::LongFloats);

        let ty: Type = parse_str("String").unwrap();
        assert_eq!(WAMRTypes::from(&ty), WAMRTypes::String);

        let ty: Type = parse_str("&str").unwrap();
        assert_eq!(WAMRTypes::from(&ty), WAMRTypes::String);

        let ty: Type = parse_str("&mut str").unwrap();
        assert_eq!(WAMRTypes::from(&ty), WAMRTypes::Pointer);
    }

    #[test]
    fn test_to_string() {
        assert_eq!(WAMRTypes::ShortIntegers.to_string(), "i");
        assert_eq!(WAMRTypes::LongIntegers.to_string(), "I");
        assert_eq!(WAMRTypes::ShortFloats.to_string(), "f");
        assert_eq!(WAMRTypes::LongFloats.to_string(), "F");
        assert_eq!(WAMRTypes::String.to_string(), "$");
        assert_eq!(WAMRTypes::Pointer.to_string(), "*");
        assert_eq!(WAMRTypes::NativeReference.to_string(), "r");
    }

    #[test]
    fn test_get_cast() {
        let ty = WAMRTypes::ShortIntegers;
        let a = quote! { a };

        assert_eq!(
            ty.get_cast(a.clone(), &parse_str("u64").unwrap())
                .to_string(),
            "let a = a as u64 ;"
        );

        let ty = WAMRTypes::LongIntegers;
        assert_eq!(
            ty.get_cast(a.clone(), &parse_str("u32").unwrap())
                .to_string(),
            "let a = a as u32 ;"
        );

        let ty = WAMRTypes::ShortFloats;
        assert_eq!(
            ty.get_cast(a.clone(), &parse_str("f64").unwrap())
                .to_string(),
            ""
        );

        let ty = WAMRTypes::LongFloats;
        assert_eq!(
            ty.get_cast(a.clone(), &parse_str("f32").unwrap())
                .to_string(),
            ""
        );

        let ty = WAMRTypes::NativeReference;
        assert_eq!(
            ty.get_cast(a.clone(), &parse_str("usize").unwrap())
                .to_string(),
            "let a : usize = unsafe { std :: mem :: transmute (a) } ;"
        );
    }
}
