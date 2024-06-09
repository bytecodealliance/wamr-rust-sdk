/*
 * Copyright (C) 2019 Intel Corporation. All rights reserved.
 * SPDX-License-Identifier: Apache-2.0 WITH LLVM-exception
 */

use crate::{instance::Instance, RuntimeError};
use std::ffi::{c_char, c_void, CStr, CString};
use std::string::String;
use wamr_sys::{wasm_runtime_addr_app_to_native, wasm_runtime_module_malloc};

pub const DEFAULT_ERROR_BUF_SIZE: usize = 128;

pub fn error_buf_to_string(&error_buf: &[c_char; DEFAULT_ERROR_BUF_SIZE]) -> String {
    let error_content: Vec<u8> = error_buf
        .map(|c| c as u8)
        .into_iter()
        .filter(|c| *c > 0)
        .collect();
    String::from_utf8(error_content).unwrap()
}

pub fn cstr_to_string(raw_cstr: *const c_char) -> String {
    let cstr = unsafe { CStr::from_ptr(raw_cstr) };
    String::from_utf8_lossy(cstr.to_bytes()).to_string()
}

pub fn exception_to_string(raw_exception: *const c_char) -> String {
    cstr_to_string(raw_exception)
}

// Convert a host string to a wasm string
// - allocate a wasm string in the wasm memory via `wasm_runtime_module_malloc()`.
//   size = strlen(host_string) + 1;
// - copy the host string to the wasm string and transform from utf-8 to ascii
// - be sure about tailing '\0'
// - return the starting index(in wasm address space)
//
// address is in native (address) space
// index is in wasm (index) space
pub fn host_string_to_wasm_string(
    instance: &Instance,
    host_string: &str,
) -> Result<u64, RuntimeError> {
    // rust string -> c string
    let host_c_string = CString::new(host_string).expect("CString::new failed");

    //TODO: optimize
    // allocation in wasm memory
    let (wasm_string_index, wasm_string_addr) = alloca_wasm_data(instance, host_string.len())?;

    // copy content. host -> wasm
    // host_c_string might be un-aligned, so we need to copy byte by byte
    let src_ptr = host_c_string.as_ptr();
    let dst_ptr = wasm_string_addr as *mut i8;

    for i in 0..host_string.len() {
        unsafe {
            let value = src_ptr.add(i).read_unaligned();
            dst_ptr.add(i).write_unaligned(value);
        }
    }

    // tailing '\0'
    unsafe {
        *((wasm_string_addr + host_string.len() as u64) as *mut i8) = 0;
    }

    Ok(wasm_string_index)
}

#[allow(dead_code)]
pub fn read_wasm_data(
    instance: &Instance,
    index: u32,
    len: usize,
) -> Result<Vec<u8>, RuntimeError> {
    println!(
        "--  --> read_wasm_data from index: 0x{:x}, len: {}",
        index, len
    );
    let p_addr: *mut c_void =
        unsafe { wasm_runtime_addr_app_to_native(instance.get_inner_instance(), index as u64) };
    if p_addr.is_null() {
        return Err(RuntimeError::OutOfBoundAccess);
    }

    let p_addr = p_addr as *mut u8;
    let mut data: Vec<u8> = Vec::with_capacity(len);
    for i in 0..len {
        let value = unsafe { p_addr.add(i).read() as u8 };
        println!("  --  --> value: 0x{:x}", value);
        data.push(value);
    }
    Ok(data)
}

// read_wasm_data() and convert it to a string
pub fn wasm_string_to_host_string(instance: &Instance, index: u32) -> Result<String, RuntimeError> {
    let p_addr: *mut c_void =
        unsafe { wasm_runtime_addr_app_to_native(instance.get_inner_instance(), index as u64) };
    if p_addr.is_null() {
        return Err(RuntimeError::OutOfBoundAccess);
    }

    let mut p_addr = p_addr as *mut u8;
    let mut data: Vec<u8> = Vec::new();
    // TBC: does it stop at '\0'? for both rust and c
    // read until '\0'
    let mut value = unsafe { p_addr.read() as u8 };
    while value != 0 {
        data.push(value);
        unsafe {
            p_addr = p_addr.add(1);
            value = p_addr.read() as u8;
        }
    }
    String::from_utf8(data)
        .map_err(|_| RuntimeError::ConvertStringError("invalid utf-8".to_string()))
}

// (wasm_index, host_addr)
pub fn alloca_wasm_data(instance: &Instance, len: usize) -> Result<(u64, u64), RuntimeError> {
    let mut wasm_data_addr: u64 = 0;
    let mut wda_void: *mut c_void = wasm_data_addr as *mut c_void;
    let p_wda_void = &mut wda_void;
    let wasm_data_index: u64 = unsafe {
        wasm_runtime_module_malloc(instance.get_inner_instance(), len as u64, p_wda_void)
    };
    wasm_data_addr = wda_void as u64;
    if wasm_data_index == 0 || wasm_data_addr == 0 {
        return Err(RuntimeError::ConvertStringError(
            ("allocate wasm data failed").to_string(),
        ));
    }
    Ok((wasm_data_index, wasm_data_addr))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{module::Module, runtime::Runtime};
    use std::ffi::CString;
    use std::path::PathBuf;

    #[test]
    fn test_error_buf_empty() {
        let error_buf = [0 as c_char; DEFAULT_ERROR_BUF_SIZE];
        let error_str = error_buf_to_string(&error_buf);
        assert_eq!(error_str.len(), 0);
        assert_eq!(error_str, "");
    }

    #[test]
    fn test_error_buf() {
        let mut error_buf = [0 as c_char; DEFAULT_ERROR_BUF_SIZE];
        error_buf[0] = 'a' as i8;
        error_buf[1] = 'b' as i8;
        error_buf[2] = 'c' as i8;

        let error_str = error_buf_to_string(&error_buf);
        assert_eq!(error_str.len(), 3);
        assert_eq!(error_str, "abc");
    }

    #[test]
    fn test_exception_to_string() {
        let exception = "it is an exception";

        let exception_cstr = CString::new(exception).expect("CString::new failed");
        let exception_str = exception_to_string(exception_cstr.as_ptr());
        assert_eq!(exception_str.len(), exception.len());
        assert_eq!(exception_str, exception);
    }

    #[test]
    fn test_host_string_to_wasm_string() {
        let runtime = Runtime::builder().use_system_allocator().build().unwrap();

        // !!! HAS TO BE a wasm32-wasi
        let mut d = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        d.push("resources/test");
        d.push("string_concat_wasm32_wasi.wasm");
        let module = Module::from_file(&runtime, d.as_path()).expect("module creation failed");

        let instance =
            Instance::new_with_args(&runtime, &module, 1024, 0).expect("instance creation failed");

        let host_string = "hi from case #test_host_string_to_wasm_string!";
        let wasm_string_index =
            host_string_to_wasm_string(&instance, host_string).expect("convert host string failed");
        assert_ne!(wasm_string_index, 0);

        let wasm_string = wasm_string_to_host_string(&instance, wasm_string_index as u32)
            .expect("get wasm string failed");
        assert_eq!(wasm_string, host_string);
    }
}
