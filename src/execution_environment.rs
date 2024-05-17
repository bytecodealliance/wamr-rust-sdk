use wamr_sys::{
    wasm_exec_env_t, wasm_runtime_get_call_stack_buf_size, wasm_runtime_get_exec_env_singleton,
};

use crate::instance::Instance;

pub struct ExecutionEnvironment(wasm_exec_env_t);

impl ExecutionEnvironment {
    pub fn from_instance(instance: Instance) -> Self {
        Self(unsafe { wasm_runtime_get_exec_env_singleton(instance.get_inner_instance()) })
    }

    /// Set user data for the current execution environment.
    /// This is useful when you want to pass some data to the host native functions.
    /// The user data can be retrieved by calling `get_user_data()`.
    ///
    /// # Safety
    /// Be careful when using this function, as it can lead to undefined behavior if misused.
    pub unsafe fn set_user_data(&self, user_data: *mut std::ffi::c_void) {
        wamr_sys::wasm_runtime_set_user_data(self.get_inner_execution_environment(), user_data);
    }

    /// Get user data for the current execution environment.
    /// This is useful when you want to pass some data to the host native functions.
    /// The user data can be set by calling `set_user_data()`.
    ///
    /// # Safety
    /// Be careful when using this function, as it can lead to undefined behavior if misused.
    pub unsafe fn get_user_data(&self) -> *mut std::ffi::c_void {
        wamr_sys::wasm_runtime_get_user_data(self.get_inner_execution_environment())
    }

    pub fn get_call_stack_buffer_size(&self) -> usize {
        unsafe {
            wasm_runtime_get_call_stack_buf_size(self.get_inner_execution_environment()) as usize
        }
    }

    pub(crate) fn get_inner_execution_environment(&self) -> wasm_exec_env_t {
        self.0
    }
}
