use std::ffi::c_void;
use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};

use wamr_rust_sdk::{
    function::Function, generate_host_function, instance::Instance, module::Module,
    runtime::Runtime, value::WasmValue, wasi_context::WasiCtxBuilder, RuntimeError,
};

#[generate_host_function]
fn now() -> i64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("Clock may have gone backwards")
        .as_millis() as i64
}

fn main() -> Result<(), RuntimeError> {
    let runtime = Runtime::builder()
        .use_system_allocator()
        .register_host_function(now)
        .build()?;

    let mut d = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    d.push("../../resources/test/basic_wasm32_unknown.wasm");  // TODO: windows can not build correct soft link
    let mut module = Module::from_file(&runtime, d.as_path())?;

    let wasi_ctx = WasiCtxBuilder::new()
        .set_pre_open_path(vec!["."], vec![])
        .build();

    module.set_wasi_context(wasi_ctx);

    let instance = Instance::new(&runtime, &module, 1024 * 64)?;

    let start_function = Function::find_export_func(&instance, "start")?;
    let stop_function = Function::find_export_func(&instance, "stop")?;

    let start_params: Vec<WasmValue> = vec![];
    let _result = start_function.call(&instance, &start_params)?;

    let stop_params: Vec<WasmValue> = vec![];
    let _result = stop_function.call(&instance, &stop_params)?;

    Ok(())
}
