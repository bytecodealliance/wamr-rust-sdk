use std::env;
use std::path::PathBuf;

use wamr_rust_sdk::{
    function::Function, generate_host_function, instance::Instance, module::Module,
    runtime::Runtime, value::WasmValue,
};

#[generate_host_function]
fn extra() -> i32 {
    100
}

#[test]
fn test_host_function() {
    let runtime = Runtime::builder()
        .use_system_allocator()
        .register_host_function(extra)
        .build()
        .unwrap();

    let mut d = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    d.push("tests/fixtures");
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
