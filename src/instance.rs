/*
 * Copyright (C) 2019 Intel Corporation. All rights reserved.
 * SPDX-License-Identifier: Apache-2.0 WITH LLVM-exception
 */

//! an instantiated module. The module is instantiated with the given imports.
//! get one via `Instance::new()`

#![allow(unused_variables)]

use alloc::string::String;
use core::{ffi::c_char, marker::PhantomData};

use wamr_sys::{
    wasm_module_inst_t, wasm_runtime_addr_app_to_native, wasm_runtime_addr_native_to_app,
    wasm_runtime_clear_exception, wasm_runtime_deinstantiate, wasm_runtime_destroy_thread_env,
    wasm_runtime_get_exception, wasm_runtime_init_thread_env, wasm_runtime_instantiate,
    wasm_runtime_module_dup_data, wasm_runtime_module_free, wasm_runtime_module_malloc,
    wasm_runtime_set_exception, wasm_runtime_validate_app_addr,
};

use crate::{
    RuntimeError, helper::DEFAULT_ERROR_BUF_SIZE, helper::error_buf_to_string, module::Module,
    runtime::Runtime,
};

#[derive(Debug)]
pub struct Instance<'module> {
    instance: wasm_module_inst_t,
    _phantom: PhantomData<Module<'module>>,
}

impl<'module> Instance<'module> {
    /// instantiate a module with stack size
    ///
    /// # Error
    ///
    /// Return `RuntimeError::CompilationError` if failed.
    pub fn new(
        runtime: &Runtime,
        module: &'module Module<'module>,
        stack_size: u32,
    ) -> Result<Self, RuntimeError> {
        Self::new_with_args(runtime, module, stack_size, 0)
    }

    /// instantiate a module with stack size and host managed heap size
    ///
    /// heap_size is used for `-nostdlib` Wasm and wasm32-unknown
    ///
    /// # Error
    ///
    /// Return `RuntimeError::CompilationError` if failed.
    pub fn new_with_args(
        _runtime: &Runtime,
        module: &'module Module<'module>,
        stack_size: u32,
        heap_size: u32,
    ) -> Result<Self, RuntimeError> {
        let init_thd_env = unsafe { wasm_runtime_init_thread_env() };
        if !init_thd_env {
            return Err(RuntimeError::InstantiationFailure(String::from(
                "thread signal env initialized failed",
            )));
        }

        let mut error_buf = [0 as c_char; DEFAULT_ERROR_BUF_SIZE];
        let instance = unsafe {
            wasm_runtime_instantiate(
                module.get_inner_module(),
                stack_size,
                heap_size,
                error_buf.as_mut_ptr(),
                error_buf.len() as u32,
            )
        };

        if instance.is_null() {
            match error_buf.len() {
                0 => {
                    return Err(RuntimeError::InstantiationFailure(String::from(
                        "instantiation failed",
                    )));
                }
                _ => {
                    return Err(RuntimeError::InstantiationFailure(error_buf_to_string(
                        &error_buf,
                    )));
                }
            }
        }

        Ok(Instance {
            instance,
            _phantom: PhantomData,
        })
    }

    pub fn get_inner_instance(&self) -> wasm_module_inst_t {
        self.instance
    }

    /// Allocate memory from the module instance's heap
    ///
    /// This allocates memory from the module instance's heap space. The returned
    /// address is an app offset (relative to module memory base), not an absolute address.
    ///
    /// # Important
    ///
    /// This function can trigger memory growth, which may invalidate existing native
    /// pointers obtained from [`addr_app_to_native`](Self::addr_app_to_native).
    ///
    /// # Parameters
    ///
    /// * `size` - The number of bytes to allocate
    ///
    /// # Returns
    ///
    /// Returns `Ok((app_offset, native_ptr))` where:
    /// * `app_offset` - The offset in the WASM app address space
    /// * `native_ptr` - The native (host) pointer to the allocated memory
    ///
    /// Returns `Err(RuntimeError)` if allocation fails
    ///
    /// # Example
    ///
    /// ```no_run
    /// # use wamr_rust_sdk::{RuntimeError, runtime::Runtime, module::Module, instance::Instance};
    /// # fn example() -> Result<(), RuntimeError> {
    /// # let runtime = Runtime::new()?;
    /// # let binary = vec![0u8; 0];
    /// # let module = Module::from_vec(&runtime, binary, "test")?;
    /// # let instance = Instance::new(&runtime, &module, 1024)?;
    /// // Allocate 100 bytes
    /// let (app_offset, native_ptr) = instance.module_malloc(100)?;
    ///
    /// // Write data to the allocated memory
    /// let data = b"Hello from host!";
    /// unsafe {
    ///     core::ptr::copy_nonoverlapping(
    ///         data.as_ptr(),
    ///         native_ptr as *mut u8,
    ///         data.len()
    ///     );
    /// }
    ///
    /// // app_offset can be passed to WASM functions
    /// // Don't forget to free when done
    /// instance.module_free(app_offset);
    /// # Ok(())
    /// # }
    /// ```
    pub fn module_malloc(&self, size: u64) -> Result<(u64, *mut core::ffi::c_void), RuntimeError> {
        let mut native_addr: *mut core::ffi::c_void = core::ptr::null_mut();
        let app_offset = unsafe {
            wasm_runtime_module_malloc(self.instance, size, &mut native_addr as *mut *mut _)
        };

        if app_offset == 0 {
            return Err(RuntimeError::InstantiationFailure(String::from(
                "module malloc failed: out of memory",
            )));
        }

        Ok((app_offset, native_addr))
    }

    /// Free memory allocated by [`module_malloc`](Self::module_malloc)
    ///
    /// # Parameters
    ///
    /// * `ptr` - The app offset returned by [`module_malloc`](Self::module_malloc)
    ///
    /// # Example
    ///
    /// ```no_run
    /// # use wamr_rust_sdk::{RuntimeError, runtime::Runtime, module::Module, instance::Instance};
    /// # fn example() -> Result<(), RuntimeError> {
    /// # let runtime = Runtime::new()?;
    /// # let binary = vec![0u8; 0];
    /// # let module = Module::from_vec(&runtime, binary, "test")?;
    /// # let instance = Instance::new(&runtime, &module, 1024)?;
    /// let (app_offset, _) = instance.module_malloc(100)?;
    /// // ... use the memory ...
    /// instance.module_free(app_offset);
    /// # Ok(())
    /// # }
    /// ```
    pub fn module_free(&self, ptr: u64) {
        unsafe {
            wasm_runtime_module_free(self.instance, ptr);
        }
    }

    /// Allocate memory and initialize it with data
    ///
    /// This is a convenience function that allocates memory from the module heap
    /// and copies the provided data into it.
    ///
    /// # Parameters
    ///
    /// * `data` - The data to copy into the allocated memory
    ///
    /// # Returns
    ///
    /// Returns `Ok(app_offset)` - the offset in the WASM app address space
    /// Returns `Err(RuntimeError)` if allocation fails
    ///
    /// # Example
    ///
    /// ```no_run
    /// # use wamr_rust_sdk::{RuntimeError, runtime::Runtime, module::Module, instance::Instance};
    /// # fn example() -> Result<(), RuntimeError> {
    /// # let runtime = Runtime::new()?;
    /// # let binary = vec![0u8; 0];
    /// # let module = Module::from_vec(&runtime, binary, "test")?;
    /// # let instance = Instance::new(&runtime, &module, 1024)?;
    /// let data = b"Hello from host!";
    /// let app_offset = instance.module_dup_data(data)?;
    /// // app_offset can be passed to WASM functions
    /// // Don't forget to free when done
    /// instance.module_free(app_offset);
    /// # Ok(())
    /// # }
    /// ```
    pub fn module_dup_data(&self, data: &[u8]) -> Result<u64, RuntimeError> {
        let app_offset = unsafe {
            wasm_runtime_module_dup_data(
                self.instance,
                data.as_ptr() as *const i8,
                data.len() as u64,
            )
        };

        if app_offset == 0 {
            return Err(RuntimeError::InstantiationFailure(String::from(
                "module dup data failed: out of memory",
            )));
        }

        Ok(app_offset)
    }

    /// Convert app address (relative offset) to native address (absolute pointer)
    ///
    /// # Important
    ///
    /// Native addresses can be invalidated on memory growth (except for shared memory).
    /// Use this function carefully and avoid caching native pointers across operations
    /// that might trigger memory growth.
    ///
    /// # Parameters
    ///
    /// * `app_offset` - The app address (offset from memory base)
    ///
    /// # Returns
    ///
    /// Returns the native pointer, or null if the address is invalid
    ///
    /// # Example
    ///
    /// ```no_run
    /// # use wamr_rust_sdk::{RuntimeError, runtime::Runtime, module::Module, instance::Instance};
    /// # fn example() -> Result<(), RuntimeError> {
    /// # let runtime = Runtime::new()?;
    /// # let binary = vec![0u8; 0];
    /// # let module = Module::from_vec(&runtime, binary, "test")?;
    /// # let instance = Instance::new(&runtime, &module, 1024)?;
    /// let (app_offset, _) = instance.module_malloc(100)?;
    /// let native_ptr = instance.addr_app_to_native(app_offset);
    /// // Use native_ptr...
    /// instance.module_free(app_offset);
    /// # Ok(())
    /// # }
    /// ```
    pub fn addr_app_to_native(&self, app_offset: u64) -> *mut core::ffi::c_void {
        unsafe { wasm_runtime_addr_app_to_native(self.instance, app_offset) }
    }

    /// Convert native address (absolute pointer) to app address (relative offset)
    ///
    /// # Parameters
    ///
    /// * `native_ptr` - The native pointer
    ///
    /// # Returns
    ///
    /// Returns the app offset, or 0 if the pointer is not in WASM memory
    ///
    /// # Example
    ///
    /// ```no_run
    /// # use wamr_rust_sdk::{RuntimeError, runtime::Runtime, module::Module, instance::Instance};
    /// # fn example() -> Result<(), RuntimeError> {
    /// # let runtime = Runtime::new()?;
    /// # let binary = vec![0u8; 0];
    /// # let module = Module::from_vec(&runtime, binary, "test")?;
    /// # let instance = Instance::new(&runtime, &module, 1024)?;
    /// let (app_offset, native_ptr) = instance.module_malloc(100)?;
    /// let converted_offset = instance.addr_native_to_app(native_ptr);
    /// assert_eq!(app_offset, converted_offset);
    /// instance.module_free(app_offset);
    /// # Ok(())
    /// # }
    /// ```
    pub fn addr_native_to_app(&self, native_ptr: *mut core::ffi::c_void) -> u64 {
        unsafe { wasm_runtime_addr_native_to_app(self.instance, native_ptr) }
    }

    /// Validate an app address range
    ///
    /// Check whether an app address range belongs to the WASM module instance's
    /// address space (heap or memory space).
    ///
    /// # Parameters
    ///
    /// * `app_offset` - The app address to validate
    /// * `size` - The size of the memory region to validate
    ///
    /// # Returns
    ///
    /// Returns `true` if the address range is valid, `false` otherwise
    ///
    /// # Example
    ///
    /// ```no_run
    /// # use wamr_rust_sdk::{RuntimeError, runtime::Runtime, module::Module, instance::Instance};
    /// # fn example() -> Result<(), RuntimeError> {
    /// # let runtime = Runtime::new()?;
    /// # let binary = vec![0u8; 0];
    /// # let module = Module::from_vec(&runtime, binary, "test")?;
    /// # let instance = Instance::new(&runtime, &module, 1024)?;
    /// let (app_offset, _) = instance.module_malloc(100)?;
    /// assert!(instance.validate_app_addr(app_offset, 100));
    /// assert!(!instance.validate_app_addr(app_offset, 1000)); // too large
    /// instance.module_free(app_offset);
    /// # Ok(())
    /// # }
    /// ```
    pub fn validate_app_addr(&self, app_offset: u64, size: u64) -> bool {
        unsafe { wasm_runtime_validate_app_addr(self.instance, app_offset, size) }
    }

    /// Get the exception info of the WASM module instance
    ///
    /// # Returns
    ///
    /// Returns `Some(String)` with the exception message if an exception occurred,
    /// `None` if no exception
    ///
    /// # Example
    ///
    /// ```no_run
    /// # use wamr_rust_sdk::{RuntimeError, runtime::Runtime, module::Module, instance::Instance};
    /// # use wamr_rust_sdk::function::Function;
    /// # use wamr_rust_sdk::value::WasmValue;
    /// # fn example() -> Result<(), RuntimeError> {
    /// # let runtime = Runtime::new()?;
    /// # let binary = vec![0u8; 0];
    /// # let module = Module::from_vec(&runtime, binary, "test")?;
    /// # let instance = Instance::new(&runtime, &module, 1024)?;
    /// # let function = Function::find_export_func(&instance, "test")?;
    /// // After calling a function that might fail
    /// match function.call(&instance, &vec![]) {
    ///     Ok(_) => println!("Success"),
    ///     Err(_) => {
    ///         if let Some(exception) = instance.get_exception() {
    ///             eprintln!("Exception: {}", exception);
    ///         }
    ///     }
    /// }
    /// # Ok(())
    /// # }
    /// ```
    pub fn get_exception(&self) -> Option<String> {
        unsafe {
            let exception_ptr = wasm_runtime_get_exception(self.instance);
            if exception_ptr.is_null() {
                return None;
            }

            let c_str = core::ffi::CStr::from_ptr(exception_ptr);
            c_str.to_str().ok().map(|s| s.to_string())
        }
    }

    /// Set exception info for the WASM module instance
    ///
    /// This is typically used in host functions to signal an error condition to the WASM module.
    ///
    /// # Parameters
    ///
    /// * `exception` - The exception message to set
    ///
    /// # Example
    ///
    /// ```no_run
    /// # use wamr_rust_sdk::{RuntimeError, runtime::Runtime, module::Module, instance::Instance};
    /// # fn example() -> Result<(), RuntimeError> {
    /// # let runtime = Runtime::new()?;
    /// # let binary = vec![0u8; 0];
    /// # let module = Module::from_vec(&runtime, binary, "test")?;
    /// # let instance = Instance::new(&runtime, &module, 1024)?;
    /// instance.set_exception("Custom error occurred");
    /// # Ok(())
    /// # }
    /// ```
    pub fn set_exception(&self, exception: &str) {
        use alloc::ffi::CString;
        let exception_cstr = CString::new(exception).expect("CString::new failed");
        unsafe {
            wasm_runtime_set_exception(self.instance, exception_cstr.as_ptr());
        }
    }

    /// Clear the exception info of the WASM module instance
    ///
    /// # Example
    ///
    /// ```no_run
    /// # use wamr_rust_sdk::{RuntimeError, runtime::Runtime, module::Module, instance::Instance};
    /// # fn example() -> Result<(), RuntimeError> {
    /// # let runtime = Runtime::new()?;
    /// # let binary = vec![0u8; 0];
    /// # let module = Module::from_vec(&runtime, binary, "test")?;
    /// # let instance = Instance::new(&runtime, &module, 1024)?;
    /// // Clear any existing exception
    /// instance.clear_exception();
    /// # Ok(())
    /// # }
    /// ```
    pub fn clear_exception(&self) {
        unsafe {
            wasm_runtime_clear_exception(self.instance);
        }
    }
}

impl Drop for Instance<'_> {
    fn drop(&mut self) {
        unsafe {
            wasm_runtime_destroy_thread_env();
            wasm_runtime_deinstantiate(self.instance);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::runtime::{Runtime, RuntimeBuilder};
    use alloc::{vec, vec::Vec};
    use wamr_sys::{
        RunningMode_Mode_Interp, RunningMode_Mode_LLVM_JIT, wasm_runtime_get_running_mode,
    };

    #[test]
    fn test_instance_new() {
        let runtime = Runtime::new().unwrap();

        // (module
        //   (func (export "add") (param i32 i32) (result i32)
        //     (local.get 0)
        //     (local.get 1)
        //     (i32.add)
        //   )
        // )
        let binary = vec![
            0x00, 0x61, 0x73, 0x6d, 0x01, 0x00, 0x00, 0x00, 0x01, 0x07, 0x01, 0x60, 0x02, 0x7f,
            0x7f, 0x01, 0x7f, 0x03, 0x02, 0x01, 0x00, 0x07, 0x07, 0x01, 0x03, 0x61, 0x64, 0x64,
            0x00, 0x00, 0x0a, 0x09, 0x01, 0x07, 0x00, 0x20, 0x00, 0x20, 0x01, 0x6a, 0x0b,
        ];
        let binary = binary.into_iter().map(|c| c as u8).collect::<Vec<u8>>();

        let module = Module::from_vec(&runtime, binary, "add");
        assert!(module.is_ok());

        let module = &module.unwrap();

        let instance = Instance::new_with_args(&runtime, module, 1024, 1024);
        assert!(instance.is_ok());

        let instance = Instance::new_with_args(&runtime, module, 1024, 0);
        assert!(instance.is_ok());

        let instance = instance.unwrap();
        assert_eq!(
            unsafe { wasm_runtime_get_running_mode(instance.get_inner_instance()) },
            if cfg!(feature = "llvmjit") {
                RunningMode_Mode_LLVM_JIT
            } else {
                RunningMode_Mode_Interp
            }
        );
    }

    #[test]
    #[ignore]
    fn test_instance_running_mode_default() {
        let runtime = Runtime::builder().use_system_allocator().build().unwrap();

        // (module
        //   (func (export "add") (param i32 i32) (result i32)
        //     (local.get 0)
        //     (local.get 1)
        //     (i32.add)
        //   )
        // )
        let binary = vec![
            0x00, 0x61, 0x73, 0x6d, 0x01, 0x00, 0x00, 0x00, 0x01, 0x07, 0x01, 0x60, 0x02, 0x7f,
            0x7f, 0x01, 0x7f, 0x03, 0x02, 0x01, 0x00, 0x07, 0x07, 0x01, 0x03, 0x61, 0x64, 0x64,
            0x00, 0x00, 0x0a, 0x09, 0x01, 0x07, 0x00, 0x20, 0x00, 0x20, 0x01, 0x6a, 0x0b,
        ];
        let binary = binary.into_iter().map(|c| c as u8).collect::<Vec<u8>>();

        let module = Module::from_vec(&runtime, binary, "");
        assert!(module.is_ok());

        let module = &module.unwrap();

        let instance = Instance::new_with_args(&runtime, module, 1024, 1024);
        assert!(instance.is_ok());

        let instance = instance.unwrap();
        assert_eq!(
            unsafe { wasm_runtime_get_running_mode(instance.get_inner_instance()) },
            if cfg!(feature = "llvmjit") {
                RunningMode_Mode_LLVM_JIT
            } else {
                RunningMode_Mode_Interp
            }
        );
    }

    #[test]
    #[ignore]
    fn test_instance_running_mode_interpreter() {
        let runtime = Runtime::builder()
            .run_as_interpreter()
            .use_system_allocator()
            .build()
            .unwrap();

        // (module
        //   (func (export "add") (param i32 i32) (result i32)
        //     (local.get 0)
        //     (local.get 1)
        //     (i32.add)
        //   )
        // )
        let binary = vec![
            0x00, 0x61, 0x73, 0x6d, 0x01, 0x00, 0x00, 0x00, 0x01, 0x07, 0x01, 0x60, 0x02, 0x7f,
            0x7f, 0x01, 0x7f, 0x03, 0x02, 0x01, 0x00, 0x07, 0x07, 0x01, 0x03, 0x61, 0x64, 0x64,
            0x00, 0x00, 0x0a, 0x09, 0x01, 0x07, 0x00, 0x20, 0x00, 0x20, 0x01, 0x6a, 0x0b,
        ];
        let binary = binary.into_iter().map(|c| c as u8).collect::<Vec<u8>>();

        let module = Module::from_vec(&runtime, binary, "add");
        assert!(module.is_ok());

        let module = &module.unwrap();

        let instance = Instance::new_with_args(&runtime, module, 1024, 1024);
        assert!(instance.is_ok());

        let instance = instance.unwrap();
        assert_eq!(
            unsafe { wasm_runtime_get_running_mode(instance.get_inner_instance()) },
            RunningMode_Mode_Interp
        );
    }

    #[test]
    fn test_module_malloc_free() {
        let runtime = Runtime::builder().use_system_allocator().build().unwrap();

        let binary = wat::parse_str(r#"
            (module
                (memory 1)
                (func (export "add") (param i32 i32) (result i32)
                    (local.get 0)
                    (local.get 1)
                    (i32.add)
                )
            )
        "#).unwrap();

        let module = Module::from_vec(&runtime, binary, "");
        assert!(module.is_ok());
        let module = &mut module.unwrap();

        // Create instance with heap size
        let instance = Instance::new_with_args(&runtime, module, 128 * 1024 * 1024, 128 * 1024 * 1024);
        assert!(instance.is_ok());
        let instance = instance.unwrap();

        // Test module_malloc
        let result = instance.module_malloc(1024);
        if let Err(e) = &result {
            eprintln!("module_malloc error: {:?}", e);
        }
        assert!(result.is_ok());
        let (app_offset, native_ptr) = result.unwrap();
        assert_ne!(app_offset, 0);
        assert!(!native_ptr.is_null());

        // Validate the allocated address
        assert!(instance.validate_app_addr(app_offset, 100));

        // Write some data to the allocated memory
        unsafe {
            let ptr = native_ptr as *mut u8;
            *ptr = 42;
            assert_eq!(*ptr, 42);
        }

        // Free the memory
        instance.module_free(app_offset);
    }

    #[test]
    fn test_module_dup_data() {
        let runtime = RuntimeBuilder::default()
            // .run_as_interpreter()
            .use_rust_allocator()
            .build()
            .unwrap();

        let binary = wat::parse_str(r#"
            (module
                (memory 1024)
                (func (export "add") (param i32 i32) (result i32)
                    (local.get 0)
                    (local.get 1)
                    (i32.add)
                )
            )
        "#).unwrap();

        let module = Module::from_vec(&runtime, binary, "").unwrap();

        // Create instance with heap size
        let instance = Instance::new_with_args(&runtime, &module, 128 * 1024 * 1024, 1024 * 1024);
        assert!(instance.is_ok());
        let instance = instance.unwrap();

        // Test module_dup_data
        let test_data = b"Hello from host!";
        let app_offset = instance.module_dup_data(test_data).unwrap();
        assert_ne!(app_offset, 0);

        // Verify the data was copied correctly
        let native_ptr = instance.addr_app_to_native(app_offset);
        assert!(!native_ptr.is_null());

        unsafe {
            let slice = core::slice::from_raw_parts(native_ptr as *const u8, test_data.len());
            assert_eq!(slice, test_data);
        }

        // Free the memory
        instance.module_free(app_offset);
    }

    #[test]
    fn test_addr_conversion() {
        let runtime = Runtime::new().unwrap();

        let binary = wat::parse_str(r#"
            (module
                (memory 1024)
                (func (export "add") (param i32 i32) (result i32)
                    (local.get 0)
                    (local.get 1)
                    (i32.add)
                )
            )
        "#).unwrap();

        let module = Module::from_vec(&runtime, binary, "test");
        assert!(module.is_ok());
        let module = &module.unwrap();

        // Create instance with heap size
        let instance = Instance::new_with_args(&runtime, &module, 128 * 1024 * 1024, 1024 * 1024);
        assert!(instance.is_ok());
        let instance = instance.unwrap();

        // Allocate memory
        let result = instance.module_malloc(100);
        assert!(result.is_ok());
        let (app_offset, native_ptr) = result.unwrap();

        // Test addr_app_to_native
        let converted_native = instance.addr_app_to_native(app_offset);
        assert_eq!(native_ptr, converted_native);

        // Test addr_native_to_app
        let converted_app = instance.addr_native_to_app(native_ptr);
        assert_eq!(app_offset, converted_app);

        // Free the memory
        instance.module_free(app_offset);
    }

    #[test]
    fn test_validate_app_addr() {
        let runtime = Runtime::new().unwrap();

        let binary = wat::parse_str(r#"
            (module
                (memory 1)
                (func (export "add") (param i32 i32) (result i32)
                    (local.get 0)
                    (local.get 1)
                    (i32.add)
                )
            )
        "#).unwrap();

        let module = Module::from_vec(&runtime, binary, "test");
        assert!(module.is_ok());
        let module = &module.unwrap();

        // Create instance with small heap size
        let instance = Instance::new_with_args(&runtime, &module, 1024, 1024);
        assert!(instance.is_ok());
        let instance = instance.unwrap();

        // Allocate memory
        let result = instance.module_malloc(100);
        assert!(result.is_ok());
        let (app_offset, _) = result.unwrap();
        println!("app_offset: {app_offset}");

        // Test validation with valid size (within allocated memory)
        assert!(instance.validate_app_addr(app_offset, 100));
        assert!(instance.validate_app_addr(app_offset, 50));

        // Test validation with invalid size (larger than allocated)
        assert!(!instance.validate_app_addr(app_offset, 1000));

        // Test validation with invalid offset
        assert!(!instance.validate_app_addr(0xFFFFFFFF, 100));

        // Test validation with offset beyond memory bounds
        let large_offset = app_offset + 1000000;
        assert!(!instance.validate_app_addr(large_offset, 100));

        // Free the memory
        instance.module_free(app_offset);
    }

    #[test]
    fn test_exception_api() {
        let runtime = Runtime::new().unwrap();

        let binary = vec![
            0x00, 0x61, 0x73, 0x6d, 0x01, 0x00, 0x00, 0x00, 0x01, 0x07, 0x01, 0x60, 0x02, 0x7f,
            0x7f, 0x01, 0x7f, 0x03, 0x02, 0x01, 0x00, 0x07, 0x07, 0x01, 0x03, 0x61, 0x64, 0x64,
            0x00, 0x00, 0x0a, 0x09, 0x01, 0x07, 0x00, 0x20, 0x00, 0x20, 0x01, 0x6a, 0x0b,
        ];
        let binary = binary.into_iter().map(|c| c as u8).collect::<Vec<u8>>();

        let module = Module::from_vec(&runtime, binary, "test");
        assert!(module.is_ok());
        let module = &module.unwrap();

        let instance = Instance::new(&runtime, module, 1024);
        assert!(instance.is_ok());
        let instance = instance.unwrap();

        // Initially no exception
        assert_eq!(instance.get_exception(), None);

        // Set an exception
        instance.set_exception("Test exception");
        assert_eq!(
            instance.get_exception(),
            Some(String::from("Exception: Test exception"))
        );

        // Clear the exception
        instance.clear_exception();
        assert_eq!(instance.get_exception(), None);

        // Set another exception
        instance.set_exception("Another error");
        assert!(instance.get_exception().is_some());
        assert!(instance
            .get_exception()
            .unwrap()
            .contains("Another error"));
    }
}
