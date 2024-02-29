/*
 * Copyright (C) 2023 Liquid Reply GmbH. All rights reserved.
 * SPDX-License-Identifier: Apache-2.0 WITH LLVM-exception
 */

use std::path::PathBuf;
use wamr_rust_sdk::{
    function::Function, instance::Instance, module::Module, runtime::Runtime, value::WasmValue,
    RuntimeError,
};

fn main() -> Result<(), RuntimeError> {
    let _runtime = Runtime::new()?;

    let mut d = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    d.push("gcd_wasm32_wasi.wasm");
    let mut module = Module::from_file(d.as_path())?;

    module.set_wasi_arg_pre_open_path(vec![String::from(".")], vec![]);

    let instance = Instance::new(&module, 1024 * 64)?;

    let function = Function::find_export_func(&instance, "gcd")?;

    let params: Vec<WasmValue> = vec![WasmValue::I32(9), WasmValue::I32(27)];
    let result = function.call(&instance, &params)?;
    assert_eq!(result, WasmValue::I32(9));

    Ok(())
}
