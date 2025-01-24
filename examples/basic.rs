use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};

use wamr_rust_sdk::{
    function::Function, generate_host_function, instance::Instance, module::Module,
    runtime::Runtime, value::WasmValue, wasi_context::WasiCtxBuilder, RuntimeError,
    sys::wasm_runtime_call_indirect,
};

#[generate_host_function]
fn now() -> i64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("Clock may have gone backwards")
        .as_millis() as i64
}

#[generate_host_function]
fn mystery(n: f32, func1: u32, func2: u32) -> f32 {
    let mut argv: Vec<u32> = Vec::with_capacity(2);

    let mut data_buffer: Vec<u32> = WasmValue::F32(n).encode();
    argv.append(&mut data_buffer);

    let func1_result = unsafe { wasm_runtime_call_indirect(exec_env, func1, 1, argv.as_mut_ptr()) };

    if !func1_result {
        println!("call func1 failed");
        return 0.0;
    }

    let n1 = argv[0] as f32;

    let func2_result = unsafe { wasm_runtime_call_indirect(exec_env, func2, 1, argv.as_mut_ptr()) };

    if !func2_result {
        println!("call func2 failed");
        return 0.0;
    }

    let n2 = argv[0] as f32;

    n1 + n2
}

fn main() -> Result<(), RuntimeError> {
    let runtime = Runtime::builder()
        .use_system_allocator()
        .register_host_function(now)
        .register_host_function(mystery)
        .build()?;

    let mut d = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    d.push("examples/modules");
    d.push("basic_wasm32_unknown.wasm");
    let mut module = Module::from_file(&runtime, d.as_path())?;

    let wasi_ctx = WasiCtxBuilder::new()
        .set_pre_open_path(vec!["."], vec![])
        .build();

    module.set_wasi_context(wasi_ctx);

    let instance = Instance::new(&runtime, &module, 1024 * 64)?;

    let start_function = Function::find_export_func(&instance, "start")?;
    let stop_function = Function::find_export_func(&instance, "stop")?;
    let calc_function = Function::find_export_func(&instance, "calculate")?;

    let start_params: Vec<WasmValue> = vec![];
    start_function.call(&instance, &start_params)?;

    let calc_params: Vec<WasmValue> = vec![WasmValue::F32(3.0)];
    if let WasmValue::F32(value) = calc_function.call(&instance, &calc_params)? {
        println!("Do a mystery calculation: input: 3, return: {}", value);
    } else {
        println!("Unexpected return value for calculate function");
    }

    let stop_params: Vec<WasmValue> = vec![];
    if let WasmValue::I64(value) = stop_function.call(&instance, &stop_params)? {
        println!("Time elapsed: {} ms", value);
    } else {
        println!("Unexpected return value for stop function");
    }

    Ok(())
}
