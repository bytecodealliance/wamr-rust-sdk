/*
 * Copyright (C) 2019 Intel Corporation. All rights reserved.
 * SPDX-License-Identifier: Apache-2.0 WITH LLVM-exception
 */

/// This is a wrapper of a host defined(Rust) function.
use std::ffi::{c_void, CString};
use std::ptr;

use wamr_sys::NativeSymbol;

#[allow(dead_code)]
#[derive(Debug)]
pub struct HostFunction {
    function_name: CString,
    function_ptr: *mut c_void,
    signature: CString,
}

impl HostFunction {
    pub fn new(name: &str, ptr: *mut c_void, signature: &str) -> Self {
        HostFunction {
            function_name: CString::new(name).unwrap(),
            function_ptr: ptr,
            signature: CString::new(signature).unwrap(),
        }
    }
}

impl<F> From<F> for HostFunction
where
    F: Fn() -> HostFunction,
{
    fn from(f: F) -> HostFunction {
        f()
    }
}

#[derive(Debug)]
pub struct HostFunctionList {
    pub module_name: CString,
    // keep ownership of the content of `native_symbols`
    host_functions: Vec<HostFunction>,
    pub native_symbols: Vec<NativeSymbol>,
}

impl HostFunctionList {
    pub fn new(module_name: &str) -> Self {
        HostFunctionList {
            module_name: CString::new(module_name).unwrap(),
            host_functions: Vec::new(),
            native_symbols: Vec::new(),
        }
    }

    pub fn register_host_function<T: Into<HostFunction>>(&mut self, function: T) {
        let host_function: HostFunction = function.into();

        self.host_functions.push(host_function);

        let last = self.host_functions.last().unwrap();
        self.native_symbols
            .push(
                NativeSymbol {
                    symbol: (&last.function_name).as_ptr(),
                    func_ptr: last.function_ptr,
                    signature: (&last.signature).as_ptr(),
                    attachment: ptr::null_mut(),
                }
            )
    }

    pub fn get_native_symbols(&mut self) -> &mut Vec<NativeSymbol> {
        &mut self.native_symbols
    }

    pub fn get_module_name(&mut self) -> &CString {
        &self.module_name
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        function::Function, instance::Instance, module::Module, runtime::Runtime, value::WasmValue,
    };
    use std::env;
    use std::path::PathBuf;

    extern "C" fn extra() -> i32 {
        100
    }

    #[test]
    #[ignore]
    fn test_host_function() {
        let runtime = Runtime::builder()
            .use_system_allocator()
            .register_host_function(HostFunction::new("extra", extra as *mut c_void, "()i"))
            .build()
            .unwrap();

        let mut d = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        d.push("resources/test");
        d.push("add_extra_wasm32_wasi.wasm");
        let module = Module::from_file(&runtime, d.as_path());
        assert!(module.is_ok());
        let module = module.unwrap();

        let instance = Instance::new(&runtime, &module, 1024 * 64);
        assert!(instance.is_ok());
        let instance: &Instance = &instance.unwrap();

        let function = Function::find_export_func(instance, "add");
        assert!(function.is_ok());
        let function = function.unwrap();

        let params: Vec<WasmValue> = vec![WasmValue::I32(8), WasmValue::I32(8)];
        let result = function.call(instance, &params);
        assert_eq!(result.unwrap(), WasmValue::I32(116));
    }
}
