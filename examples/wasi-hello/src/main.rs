/*
 * Copyright (C) 2023 Liquid Reply GmbH. All rights reserved.
 * SPDX-License-Identifier: Apache-2.0 WITH LLVM-exception
 */

use std::path::PathBuf;
use wamr_rust_sdk::{
    function::Function, instance::Instance, module::Module, runtime::Runtime, value::WasmValue,
    wasi_context::WasiCtxBuilder, RuntimeError,
};

fn main() -> Result<(), RuntimeError> {
    let runtime = Runtime::new()?;

    let mut d = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    d.push("gcd_wasm32_wasi.wasm");
    let mut module = Module::from_file(&runtime, d.as_path())?;

    let wasi_ctx = WasiCtxBuilder::new()
        .set_pre_open_path(vec!["."], vec![])
        .build();

    module.set_wasi_context(wasi_ctx);

    let instance = Instance::new(&runtime, &module, 1024 * 64)?;

    let function = Function::find_export_func(&instance, "gcd")?;

    let params: Vec<WasmValue> = vec![WasmValue::I32(9), WasmValue::I32(27)];
    let result = function.call(&instance, &params)?;
    assert_eq!(result, WasmValue::I32(9));

    Ok(())
}
